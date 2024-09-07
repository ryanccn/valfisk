use std::sync::Arc;

use color_eyre::eyre::{Report, Result, WrapErr as _};
use tracing::{info, warn};

use poise::{serenity_prelude as serenity, Framework, FrameworkContext, FrameworkOptions};
use storage::Storage;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::utils::Pluralize as _;

#[derive(Debug)]
pub struct Data {
    storage: Option<Storage>,
}

pub type Context<'a> = poise::Context<'a, Data, Report>;

mod api;
mod commands;
mod handlers;
mod reqwest_client;
mod starboard;
mod storage;
mod template_channel;
mod utils;

#[tracing::instrument(skip_all)]
async fn event_handler(
    ctx: FrameworkContext<'_, Data, Report>,
    ev: &serenity::FullEvent,
) -> Result<()> {
    use serenity::{FullEvent, Timestamp};

    let data = ctx.user_data();

    match ev {
        FullEvent::Ready { data_about_bot } => {
            info!("Connected to Discord as {}", data_about_bot.user.tag());

            let commands = &ctx.options().commands;
            poise::builtins::register_globally(&ctx.serenity_context.http, commands).await?;

            info!(
                "Registered {} {}",
                commands.len(),
                "command".pluralize(commands.len())
            );

            commands::restore_presence(ctx.serenity_context, &ctx.user_data()).await?;
        }

        FullEvent::Message { new_message } => {
            handlers::handle_message(new_message, ctx.serenity_context, &data).await?;
        }

        FullEvent::MessageUpdate { event, .. } => {
            use storage::log::MessageLog;

            let timestamp = event.edited_timestamp.unwrap_or_else(Timestamp::now);

            if let Some(storage) = &data.storage {
                let prev = storage.get_message_log(&event.id.to_string()).await?;

                let content = event.content.clone();
                let author = event.author.as_ref().map(|a| a.id);

                storage
                    .set_message_log(
                        &event.id.to_string(),
                        &MessageLog::new(content.as_ref().map(|s| s.to_string()), author),
                    )
                    .await?;

                handlers::log::edit(
                    ctx.serenity_context,
                    (&event.id, &event.channel_id, &event.guild_id),
                    &author,
                    &prev.and_then(|p| p.content),
                    &content
                        .as_ref()
                        .map_or("*Unknown*".to_owned(), |s| s.to_string()),
                    &timestamp,
                )
                .await?;
            }
        }

        FullEvent::MessageDelete {
            deleted_message_id,
            channel_id,
            guild_id,
        } => {
            starboard::handle_deletion(ctx.serenity_context, &data, deleted_message_id, channel_id)
                .await?;

            let timestamp = Timestamp::now();

            if let Some(storage) = &data.storage {
                let prev = storage
                    .get_message_log(&deleted_message_id.to_string())
                    .await?;

                handlers::log::delete(
                    ctx.serenity_context,
                    (deleted_message_id, channel_id, guild_id),
                    &prev.as_ref().and_then(|p| p.author),
                    &prev.and_then(|p| p.content),
                    &timestamp,
                )
                .await?;

                storage
                    .del_message_log(&deleted_message_id.to_string())
                    .await?;
            }
        }

        FullEvent::ReactionAdd { add_reaction } => {
            let message = add_reaction.message(ctx.serenity_context).await?;
            starboard::handle(ctx.serenity_context, &data, &message).await?;
        }

        FullEvent::ReactionRemove { removed_reaction } => {
            let message = removed_reaction.message(ctx.serenity_context).await?;
            starboard::handle(ctx.serenity_context, &data, &message).await?;
        }

        FullEvent::ReactionRemoveAll {
            removed_from_message_id,
            channel_id,
        } => {
            let message = channel_id
                .message(ctx.serenity_context, *removed_from_message_id)
                .await?;
            starboard::handle(ctx.serenity_context, &data, &message).await?;
        }

        FullEvent::ReactionRemoveEmoji { removed_reactions } => {
            let message = removed_reactions.message(ctx.serenity_context).await?;
            starboard::handle(ctx.serenity_context, &data, &message).await?;
        }

        FullEvent::GuildMemberAddition { new_member } => {
            handlers::log::member_join(ctx.serenity_context, &new_member.user).await?;
        }

        FullEvent::GuildMemberRemoval {
            user,
            member_data_if_available,
            ..
        } => {
            handlers::log::member_leave(ctx.serenity_context, user, member_data_if_available)
                .await?;
        }

        FullEvent::PresenceUpdate { new_data, .. } => {
            if new_data.guild_id.map(|g| g.to_string()) == std::env::var("GUILD_ID").ok() {
                let mut store = api::PRESENCE_STORE.write().unwrap();
                store.insert(
                    new_data.user.id,
                    api::ValfiskPresenceData::from_presence(new_data),
                );
                drop(store);
            }
        }

        &_ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "valfisk,warn,error");
    };

    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_error::ErrorLayer::default())
        .init();

    #[cfg(debug_assertions)]
    {
        if let Ok(dotenv_path) = dotenvy::dotenv() {
            warn!(
                "Loaded environment variables from {}",
                dotenv_path.to_string_lossy()
            );
        }
    }

    let token = std::env::var("DISCORD_TOKEN")
        .wrap_err_with(|| "Could not obtain DISCORD_TOKEN from environment!")?;

    let storage = if let Ok(redis_url) = std::env::var("REDIS_URL") {
        let client = redis::Client::open(redis_url)?;
        Some(Storage::from(client))
    } else {
        None
    };

    let mut client = serenity::Client::builder(&token, serenity::GatewayIntents::all())
        .framework(Framework::new(FrameworkOptions {
            commands: commands::to_vec(),
            event_handler: |ctx, ev| Box::pin(event_handler(ctx, ev)),
            on_error: |err| Box::pin(handlers::handle_error(err)),
            ..Default::default()
        }))
        .data(Arc::new(Data { storage }))
        .await?;

    let client_http_2 = client.http.clone();

    tokio::select! {
        result = client.start() => { result.map_err(color_eyre::eyre::Error::from) },
        result = api::serve(client_http_2) => { result },
        _ = tokio::signal::ctrl_c() => {
            warn!("Interrupted with SIGINT, exiting");
            std::process::exit(130);
        },
    }
}
