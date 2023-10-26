use anyhow::{Error, Result};
use owo_colors::OwoColorize;

use poise::{
    serenity_prelude::{Client, FullEvent, GatewayIntents},
    Framework, FrameworkOptions,
};

use crate::utils::Pluralize;

pub struct Data {}
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod handlers;
mod presence_api;
mod reqwest_client;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    let mut client = Client::builder(std::env::var("DISCORD_TOKEN")?, GatewayIntents::all())
        .framework(Framework::new(
            FrameworkOptions {
                commands: commands::vec(),
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
                                    let mut presence_store =
                                        presence_api::PRESENCE_STORE.lock().await;
                                    presence_store.insert(
                                        new_data.user.id,
                                        presence_api::ValfiskPresenceData::from_presence(new_data),
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
                    let tag = ready.user.tag();
                    println!("{} to Discord as {}", "Connected".green(), tag.cyan());

                    let commands = &framework.options().commands;

                    poise::builtins::register_globally(&ctx, commands).await?;
                    println!(
                        "{} {} {}",
                        "Registered".blue(),
                        commands.len(),
                        "command".pluralize(commands.len())
                    );

                    Ok(Data {})
                })
            },
        ))
        .await?;

    tokio::select! {
        result = client.start() => { result.map_err(anyhow::Error::from) },
        result = presence_api::serve() => { result },
        _ = tokio::signal::ctrl_c() => {
            println!("{} with SIGINT, exiting", "Interrupted".magenta());
            std::process::exit(130);
        },
    }
}
