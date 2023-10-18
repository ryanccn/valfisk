use anyhow::Result;
use poise::serenity_prelude as serenity;

mod github_expansion;

pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    tokio::try_join!(github_expansion::handle(message, ctx))?;

    Ok(())
}
