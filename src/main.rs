// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::{env, sync::Arc};

use eyre::{Report, Result};
use tracing::{info, warn};

use poise::{serenity_prelude as serenity, Framework, FrameworkContext, FrameworkOptions};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::config::CONFIG;
use crate::safe_browsing::SafeBrowsing;
use crate::storage::Storage;
use crate::utils::Pluralize as _;

mod api;
mod commands;
mod config;
mod handlers;
mod intelligence;
mod reqwest_client;
mod safe_browsing;
mod schedule;
mod starboard;
mod storage;
mod template_channel;
mod utils;

#[derive(Debug)]
pub struct Data {
    storage: Option<Storage>,
    safe_browsing: Option<SafeBrowsing>,
}

impl Data {
    fn new() -> Result<Self> {
        let storage = if let Some(redis_url) = &CONFIG.redis_url {
            let client = redis::Client::open(redis_url.clone())?;
            Some(Storage::from(client))
        } else {
            None
        };

        let safe_browsing = CONFIG
            .safe_browsing_api_key
            .as_ref()
            .map(|key| SafeBrowsing::new(key));

        Ok(Self {
            storage,
            safe_browsing,
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Report>;

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
            Box::pin(handlers::handle_message(
                new_message,
                ctx.serenity_context,
                &data,
            ))
            .await?;
        }

        FullEvent::MessageUpdate { event, .. } => {
            if event.guild_id == CONFIG.guild_id {
                use storage::log::MessageLog;

                let timestamp = event.edited_timestamp.unwrap_or_else(Timestamp::now);

                if let Some(storage) = &data.storage {
                    let prev = storage.get_message_log(&event.id.to_string()).await?;

                    let content = event.content.clone();
                    let author = event.author.as_ref().map(|a| a.id);
                    let attachments = event
                        .attachments
                        .as_ref()
                        .map(|a| a.to_vec())
                        .unwrap_or_default();

                    storage
                        .set_message_log(
                            &event.id.to_string(),
                            &MessageLog::new(
                                content.as_ref().map(|s| s.to_string()),
                                author,
                                attachments.clone(),
                            ),
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
                        &attachments,
                        &timestamp,
                    )
                    .await?;
                }
            }
        }

        FullEvent::MessageDelete {
            deleted_message_id,
            channel_id,
            guild_id,
        } => {
            if *guild_id == CONFIG.guild_id {
                starboard::handle_deletion(
                    ctx.serenity_context,
                    &data,
                    deleted_message_id,
                    channel_id,
                )
                .await?;

                let timestamp = Timestamp::now();

                if let Some(storage) = &data.storage {
                    let prev = storage
                        .get_message_log(&deleted_message_id.to_string())
                        .await?;

                    handlers::log::delete(
                        ctx.serenity_context,
                        (deleted_message_id, channel_id, guild_id),
                        &prev,
                        &timestamp,
                    )
                    .await?;

                    storage
                        .del_message_log(&deleted_message_id.to_string())
                        .await?;
                }
            }
        }

        FullEvent::ReactionAdd { add_reaction } => {
            if add_reaction.guild_id == CONFIG.guild_id {
                let message = add_reaction.message(ctx.serenity_context).await?;
                starboard::handle(ctx.serenity_context, &data, &message).await?;
            }
        }

        FullEvent::ReactionRemove { removed_reaction } => {
            if removed_reaction.guild_id == CONFIG.guild_id {
                let message = removed_reaction.message(ctx.serenity_context).await?;
                starboard::handle(ctx.serenity_context, &data, &message).await?;
            }
        }

        FullEvent::ReactionRemoveAll {
            removed_from_message_id,
            channel_id,
        } => {
            if Some(
                channel_id
                    .to_guild_channel(&ctx.serenity_context, None)
                    .await?
                    .guild_id,
            ) == CONFIG.guild_id
            {
                let message = channel_id
                    .message(ctx.serenity_context, *removed_from_message_id)
                    .await?;
                starboard::handle(ctx.serenity_context, &data, &message).await?;
            }
        }

        FullEvent::ReactionRemoveEmoji { removed_reactions } => {
            if removed_reactions.guild_id == CONFIG.guild_id {
                let message = removed_reactions.message(ctx.serenity_context).await?;
                starboard::handle(ctx.serenity_context, &data, &message).await?;
            }
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
            if new_data.guild_id == CONFIG.guild_id {
                let mut store = api::PRESENCE_STORE.write().await;
                store.insert(new_data.user.id, new_data.into());
                drop(store);
            }
        }

        FullEvent::GuildCreate { guild, .. } => {
            if Some(guild.id) != CONFIG.guild_id {
                guild.id.leave(&ctx.serenity_context.http).await?;
            }
        }

        &_ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "valfisk=info,warn,error");
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
                dotenv_path.display()
            );
        }
    }

    // Preload config from environment
    let _ = *CONFIG;

    if CONFIG.redis_url.is_none() {
        warn!("`REDIS_URL` is not configured, some features may be disabled");
    }

    let data = Arc::new(Data::new()?);

    if let Some(safe_browsing) = &data.safe_browsing {
        safe_browsing.update().await?;
    }

    let mut client =
        serenity::Client::builder(&CONFIG.discord_token, serenity::GatewayIntents::all())
            .framework(Framework::new(FrameworkOptions {
                commands: commands::to_vec(),
                event_handler: |ctx, ev| Box::pin(event_handler(ctx, ev)),
                on_error: |err| Box::pin(handlers::handle_error(err)),
                ..Default::default()
            }))
            .data(data.clone())
            .await?;

    tokio::select! {
        result = api::serve(client.http.clone()) => { result },
        result = schedule::start(client.http.clone(), data.clone()) => { result },
        result = client.start() => { result.map_err(eyre::Report::from) },
        _ = tokio::signal::ctrl_c() => {
            warn!("Interrupted with SIGINT, exiting");
            std::process::exit(130);
        },
    }
}
