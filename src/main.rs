#![warn(clippy::all, clippy::pedantic, clippy::perf)]
#![allow(
    clippy::unreadable_literal,
    clippy::module_name_repetitions,
    clippy::unused_async
)]
#![forbid(unsafe_code)]

use color_eyre::eyre::{Context as EyreContext, Error, Result};
use log::{error, info, warn};

use poise::{
    serenity_prelude::{Client, FullEvent, GatewayIntents},
    Framework, FrameworkOptions,
};

use crate::utils::Pluralize;

pub struct Data {
    pub redis: Option<redis::Client>,
}
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod api;
mod commands;
mod handlers;
mod reqwest_client;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "valfisk,warn,error");
    };

    color_eyre::install()?;
    env_logger::init();

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
        .context("Could not obtain DISCORD_TOKEN from environment!")?;

    let mut client = Client::builder(token, GatewayIntents::all())
        .framework(Framework::new(
            FrameworkOptions {
                commands: commands::to_vec(),
                event_handler: |ctx, ev, _, _| {
                    Box::pin(async move {
                        match ev {
                            FullEvent::Message { new_message } => {
                                handlers::handle_message(new_message, ctx).await?;
                            }

                            FullEvent::PresenceUpdate { new_data, .. } => {
                                if new_data.guild_id.map(|g| g.to_string())
                                    == std::env::var("GUILD_ID").ok()
                                {
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
                    })
                },
                on_error: |err| {
                    Box::pin(async move {
                        handlers::handle_error(&err).await;
                    })
                },
                ..Default::default()
            },
            |ctx, ready, framework| {
                Box::pin(async move {
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

                        if let Err(err) = commands::restore_presence(ctx, &client).await {
                            error!("{err}");
                        };

                        Ok(Data {
                            redis: Some(client),
                        })
                    } else {
                        Ok(Data { redis: None })
                    }
                })
            },
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
