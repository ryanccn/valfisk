// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::analytics;

mod autoreply;
pub mod code_expansion;
pub mod config;
mod dm;
mod error;
mod honeypot;
pub mod intelligence;
pub mod log;
mod safe_browsing;
pub mod starboard;

pub use error::error;

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn message_guild(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    let guild_config = if let Some(guild_id) = message.guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        Some(storage.get_config(guild_id).await?)
    } else {
        None
    };

    if safe_browsing::handle(ctx, guild_config.as_ref(), message).await? {
        return Ok(());
    }

    if honeypot::handle(ctx, guild_config.as_ref(), message).await? {
        return Ok(());
    }

    let results = tokio::join!(
        log::handle_message(ctx, guild_config.as_ref(), message),
        autoreply::handle(ctx, message),
        code_expansion::handle_message(ctx, message),
        intelligence::handle(ctx, message),
    );

    for result in [results.0, results.1, results.2, results.3] {
        if let Err(err) = result {
            tracing::error!("{err:?}");
        }
    }

    analytics::send_event("message_v1", message.guild_id);

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn message_dm(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    dm::handle(ctx, message).await?;
    Ok(())
}
