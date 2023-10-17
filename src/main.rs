use anyhow::{Error, Result};
use owo_colors::OwoColorize;
use poise::{
    serenity_prelude::{Client, GatewayIntents},
    Framework, FrameworkOptions,
};

use crate::utils::Pluralize;

pub struct Data {}
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod reqwest_client;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    let mut client = Client::builder(
        std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"),
        GatewayIntents::all(),
    )
    .framework(Framework::new(
        FrameworkOptions {
            commands: commands::vec(),
            ..Default::default()
        },
        |ctx, ready, framework| {
            Box::pin(async move {
                let tag = ready.user.tag();
                println!("{} to Discord ({})", "Connected".green(), tag.cyan());

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

    client.start().await?;

    Ok(())
}
