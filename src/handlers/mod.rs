use eyre::Result;
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
        code_expansion::handle(ctx, message),
        autoreply::handle(ctx, data, message),
        log::handle_message(ctx, data, message),
        intelligence::handle(ctx, message),
        dm::handle(ctx, message)
    )?;

    Ok(())
}
