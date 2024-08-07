use color_eyre::eyre::{Report, Result, WrapErr as _};
use tracing::{info, warn};

use poise::{serenity_prelude as serenity, Framework, FrameworkOptions};
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
    ctx: &serenity::Context,
    ev: &serenity::FullEvent,
    // framework: poise::FrameworkContext<'_, Data, Report>,
    data: &Data,
) -> Result<()> {
    use serenity::FullEvent;

    match ev {
        FullEvent::Message { new_message } => {
            handlers::handle_message(new_message, ctx, data).await?;
        }

        FullEvent::ReactionAdd { add_reaction } => {
            let message = add_reaction.message(&ctx).await?;
            starboard::handle(ctx, data, &message).await?;
        }

        FullEvent::ReactionRemove { removed_reaction } => {
            let message = removed_reaction.message(&ctx).await?;
            starboard::handle(ctx, data, &message).await?;
        }

        FullEvent::ReactionRemoveAll {
            removed_from_message_id,
            channel_id,
        } => {
            let message = channel_id.message(&ctx, removed_from_message_id).await?;
            starboard::handle(ctx, data, &message).await?;
        }

        FullEvent::ReactionRemoveEmoji { removed_reactions } => {
            let message = removed_reactions.message(&ctx).await?;
            starboard::handle(ctx, data, &message).await?;
        }

        FullEvent::MessageDelete {
            deleted_message_id,
            channel_id,
            ..
        } => {
            starboard::handle_deletion(ctx, data, deleted_message_id, channel_id).await?;
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

#[tracing::instrument(skip_all)]
async fn setup(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<Data, Report>,
) -> Result<Data> {
    info!("Connected to Discord as {}", ready.user.tag());

    let commands = &framework.options().commands;

    poise::builtins::register_globally(&ctx, commands).await?;
    info!(
        "Registered {} {}",
        commands.len(),
        "command".pluralize(commands.len())
    );

    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        let client = redis::Client::open(redis_url)?;
        let storage = Storage::from(client);

        commands::restore_presence(ctx, &storage).await?;

        Ok(Data {
            storage: Some(storage),
        })
    } else {
        Ok(Data { storage: None })
    }
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

    let mut client = serenity::Client::builder(token, serenity::GatewayIntents::all())
        .framework(Framework::new(
            FrameworkOptions {
                commands: commands::to_vec(),
                event_handler: |ctx, ev, _, data| Box::pin(event_handler(ctx, ev, data)),
                on_error: |err| {
                    Box::pin(async move {
                        handlers::handle_error(&err).await;
                    })
                },
                ..Default::default()
            },
            |ctx, ready, framework| Box::pin(setup(ctx, ready, framework)),
        ))
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
