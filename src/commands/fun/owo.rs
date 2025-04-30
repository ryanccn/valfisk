// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use crate::Context;

/// owo
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, install_context = "Guild | User")]
pub async fn owo(ctx: Context<'_>) -> Result<()> {
    ctx.say("owo").await?;

    Ok(())
}
