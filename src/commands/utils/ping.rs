// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use crate::Context;

/// Pong!
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say(format!(
        "Pong! {:#}",
        jiff::SignedDuration::try_from(ctx.ping().unwrap_or_default())
            .ok()
            .and_then(|d| d.round(jiff::Unit::Millisecond).ok())
            .unwrap_or_default()
    ))
    .await?;

    Ok(())
}
