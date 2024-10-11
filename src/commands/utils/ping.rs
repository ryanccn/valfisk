// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use crate::Context;

/// Pong!
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    let ping = humantime::format_duration(ctx.ping().await);
    ctx.say(format!("Pong! `{ping}`")).await?;

    Ok(())
}
