// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

mod autoreply;
mod code_expansion;
mod dm;
mod error_handling;
mod intelligence;
pub mod log;
mod safe_browsing;

pub use error_handling::handle_error;

use crate::Data;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle_message(
    message: &serenity::Message,
    ctx: &serenity::Context,
    data: &Data,
) -> Result<()> {
    if safe_browsing::handle(ctx, data, message).await? {
        return Ok(());
    }

    tokio::try_join!(
        log::handle_message(ctx, data, message),
        autoreply::handle(ctx, data, message),
        code_expansion::handle(ctx, message),
        intelligence::handle(ctx, message),
        dm::handle(ctx, message)
    )?;

    Ok(())
}
