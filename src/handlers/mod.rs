use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

mod error_handling;
mod github_expansion;

pub use error_handling::handle_error;

pub async fn handle_message(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    tokio::try_join!(github_expansion::handle(message, ctx))?;

    Ok(())
}
