use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

mod code_expansion;
mod error_handling;

pub use error_handling::handle_error;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle_message(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    tokio::try_join!(code_expansion::handle(message, ctx))?;

    Ok(())
}
