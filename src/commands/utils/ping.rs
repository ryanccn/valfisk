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
        "Pong! `{}`",
        humantime::format_duration(ctx.ping().await.unwrap_or_default())
    ))
    .await?;

    Ok(())
}
