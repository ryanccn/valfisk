#![warn(clippy::all, clippy::pedantic, clippy::perf)]
#![allow(
    clippy::unreadable_literal,
    clippy::module_name_repetitions,
    clippy::unused_async
)]
#![forbid(unsafe_code)]

use anyhow::{Context as AnyhowContext, Error, Result};
use owo_colors::OwoColorize;

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
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .context("Could not obtain DISCORD_TOKEN from environment!")?;

    let mut client = Client::builder(token, GatewayIntents::all())
        .framework(Framework::new(
            FrameworkOptions {
                commands: commands::to_vec(),
                event_handler: |ev, _, _| {
                    Box::pin(async move {
                        match ev {
                            FullEvent::Message { new_message, ctx } => {
                                handlers::handle_message(new_message, ctx).await?;
                            }

                            FullEvent::PresenceUpdate { new_data, .. } => {
                                if new_data.guild_id.map(|g| g.to_string())
                                    == std::env::var("GUILD_ID").ok()
                                {
                                    let mut presence_store = api::PRESENCE_STORE.lock().unwrap();
                                    presence_store.insert(
                                        new_data.user.id,
                                        api::ValfiskPresenceData::from_presence(new_data),
                                    );
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
                    println!(
                        "{} to Discord as {}",
                        "Connected".green(),
                        ready.user.tag().cyan()
                    );

                    let commands = &framework.options().commands;

                    poise::builtins::register_globally(&ctx, commands).await?;
                    println!(
                        "{} {} {}",
                        "Registered".blue(),
                        commands.len(),
                        "command".pluralize(commands.len())
                    );

                    if let Ok(redis_url) = std::env::var("REDIS_URL") {
                        let client = redis::Client::open(redis_url)?;

                        if let Err(err) = commands::restore_presence(ctx, &client).await {
                            eprintln!("{err}");
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

    tokio::select! {
        result = client.start() => { result.map_err(anyhow::Error::from) },
        result = api::serve() => { result },
        _ = tokio::signal::ctrl_c() => {
            println!("{} with SIGINT, exiting", "Interrupted".magenta());
            std::process::exit(130);
        },
    }
}
