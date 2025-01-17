// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Send a message in the current channel
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
pub async fn say(
    ctx: Context<'_>,
    #[description = "Text to send in the current channel"] content: String,
    #[description = "User to send DM to"] dm: Option<serenity::UserId>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Some(user) = dm {
        user.create_dm_channel(&ctx).await?.id
    } else {
        ctx.channel_id()
    }
    .send_message(
        ctx.http(),
        serenity::CreateMessage::default().content(content),
    )
    .await?;

    ctx.say("Done!").await?;

    Ok(())
}
