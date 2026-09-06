// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, commands::moderation::ModerationAction};

/// Warn a user
#[tracing::instrument(skip(ctx, user), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "The user to warn"] user: serenity::User,
    #[description = "Reason for the warn"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let mut action = ModerationAction::new(ctx, "Warn", user.id, 0xfacc15).await?;

    let warn_count = if let Some(storage) = &ctx.data().storage {
        Some(storage.incr_warn_count(user.id, action.guild().id).await?)
    } else {
        None
    };

    if let Some(reason) = &reason {
        action = action.field("Reason", reason);
    }

    if let Some(warn_count) = &warn_count {
        action = action.field("Warn count", warn_count);
    }

    action = action.notify(&user, dm.unwrap_or(true)).await;
    action.log().await?;
    action.reply().await
}

/// Reset a user's warn count to zero
#[tracing::instrument(skip(ctx), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    rename = "warn-reset",
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn warn_reset(
    ctx: Context<'_>,
    #[description = "The user to reset warns for"] user: serenity::User,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let action = ModerationAction::new(ctx, "Warn reset", user.id, 0xfacc15).await?;

    if let Some(storage) = &ctx.data().storage {
        storage.del_warn_count(user.id, action.guild().id).await?;
    }

    let mut action = action.field("Warn count", 0);

    action = action.notify(&user, dm.unwrap_or(true)).await;
    action.log().await?;
    action.reply().await
}
