// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

mod autoreply;
pub mod code_expansion;
pub mod config;
mod dm;
mod error;
mod intelligence;
pub mod log;
mod safe_browsing;
pub mod starboard;

pub use error::error;

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn message_guild(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if safe_browsing::handle(ctx, message).await? {
        return Ok(());
    }

    tokio::try_join!(
        log::handle_message(ctx, message),
        autoreply::handle(ctx, message),
        code_expansion::handle_message(ctx, message),
        intelligence::handle(ctx, message)
    )?;

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn message_dm(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    dm::handle(ctx, message).await?;
    Ok(())
}
