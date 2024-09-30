use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

mod autoreply;
mod code_expansion;
mod dm;
mod error_handling;
mod intelligence;
pub mod log;

pub use error_handling::handle_error;

use crate::Data;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle_message(
    message: &serenity::Message,
    ctx: &serenity::Context,
    data: &Data,
) -> Result<()> {
    tokio::try_join!(
        code_expansion::handle(message, ctx),
        autoreply::handle(message, ctx, data),
        log::handle_message(message, data),
        intelligence::handle(message, ctx),
        dm::handle(message, ctx)
    )?;

    Ok(())
}
