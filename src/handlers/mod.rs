use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

mod code_expansion;
mod error_handling;
pub mod log;

pub use error_handling::handle_error;

use crate::Data;

pub async fn handle_message(
    message: &serenity::Message,
    ctx: &serenity::Context,
    data: &Data,
) -> Result<()> {
    tokio::try_join!(
        code_expansion::handle(message, ctx),
        log::handle_message(message, data)
    )?;

    Ok(())
}
