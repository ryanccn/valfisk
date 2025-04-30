// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use crate::Context;

/// Pong!
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, install_context = "Guild | User")]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    let ping = match ctx.ping().await {
        Some(d) => humantime::format_duration(d).to_string(),
        None => "?".to_string(),
    };

    ctx.say(format!("Pong! `{ping}`")).await?;

    Ok(())
}
