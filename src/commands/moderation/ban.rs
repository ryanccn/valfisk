// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, commands::moderation::ModerationAction, utils};

/// Ban a user
#[tracing::instrument(skip(ctx, user), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The user to ban"] user: serenity::User,
    #[description = "Reason for the ban"] reason: Option<String>,

    #[description = "Days of messages to delete (default: 0)"]
    #[min = 0]
    #[max = 7]
    delete_message_days: Option<u32>,

    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    let delete_message_days = delete_message_days.unwrap_or(0);

    ctx.defer_ephemeral().await?;

    let mut action = ModerationAction::new(ctx, "Ban", user.id, 0xda77f2).await?;

    let extra_message = action
        .guild_config()
        .and_then(|c| c.moderation_extra_message_ban.clone());

    if let Some(reason) = utils::option_strings(reason.as_deref(), extra_message.as_deref()) {
        action = action.field("Reason", reason);
    }

    action = action.field("Days of messages deleted", delete_message_days);

    action = action.notify(&user, dm.unwrap_or(true)).await;
    action.log().await?;

    action
        .guild()
        .id
        .ban(
            ctx.http(),
            user.id,
            delete_message_days * 86400,
            reason.as_deref(),
        )
        .await?;

    action.reply().await
}
