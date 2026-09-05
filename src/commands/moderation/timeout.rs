// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, commands::moderation::ModerationAction, utils};

/// Timeout a user
#[tracing::instrument(skip(ctx, user), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "The user to timeout"] user: serenity::User,
    #[description = "Duration of timeout"] duration: String,
    #[description = "Reason for the timeout"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let Ok(duration) = humantime::parse_duration(&duration) else {
        ctx.say("Invalid duration provided!").await?;
        return Ok(());
    };

    if duration > std::time::Duration::from_hours(28 * 24) {
        ctx.say("Duration must not exceed 28 days!").await?;
        return Ok(());
    }

    let end = chrono::Utc::now() + duration;

    let mut action = ModerationAction::new(ctx, "Timeout", user.id, 0x9775fa).await?;

    let extra_message = action
        .guild_config()
        .and_then(|c| c.moderation_extra_message_timeout.clone());

    if let Some(reason) = utils::option_strings(reason.as_deref(), extra_message.as_deref()) {
        action = action.field("Reason", reason);
    }

    action = action.field("Duration", humantime::format_duration(duration));
    action = action.notify(&user, dm.unwrap_or(true)).await;

    action.log().await?;

    let mut edit_member = serenity::EditMember::default().disable_communication_until(end.into());

    if let Some(reason) = &reason {
        edit_member = edit_member.audit_log_reason(reason);
    }

    action
        .guild()
        .id
        .member(&ctx, user.id)
        .await?
        .edit(ctx.http(), edit_member)
        .await?;

    action.reply().await
}
