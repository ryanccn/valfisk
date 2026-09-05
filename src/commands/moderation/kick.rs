// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, commands::moderation::ModerationAction, utils};

/// Kick a user
#[tracing::instrument(skip(ctx, user), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The user to kick"] user: serenity::User,
    #[description = "Reason for the kick"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let mut action = ModerationAction::new(ctx, "Kick", user.id, 0xf783ac).await?;

    let extra_message = action
        .guild_config()
        .and_then(|c| c.moderation_extra_message_kick.clone());

    if let Some(reason) = utils::option_strings(reason.as_deref(), extra_message.as_deref()) {
        action = action.field("Reason", reason);
    }

    action = action.notify(&user, dm.unwrap_or(true)).await;

    action.log().await?;

    action
        .guild()
        .id
        .kick(ctx.http(), user.id, reason.as_deref())
        .await?;

    action.reply().await
}
