// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, utils};

/// Ban a member
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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
    #[description = "The member to ban"] member: serenity::Member,
    #[description = "Reason for the ban"] reason: Option<String>,

    #[description = "Days of messages to delete (default: 0)"]
    #[min = 0]
    #[max = 7]
    delete_message_days: Option<u32>,

    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    member
        .ban(
            ctx.http(),
            delete_message_days.unwrap_or(0),
            reason.as_deref(),
        )
        .await?;

    let mut dm_embed = serenity::CreateEmbed::default()
        .title("Ban")
        .field(
            "User",
            utils::serenity::format_mentionable(Some(member.user.id)),
            false,
        )
        .color(0xda77f2)
        .timestamp(serenity::Timestamp::now());

    if let Some(reason) = &reason {
        dm_embed = dm_embed.field("Reason", reason, false);
    }

    dm_embed = dm_embed.field(
        "Days of messages deleted",
        delete_message_days.unwrap_or(0).to_string(),
        false,
    );

    if dm.unwrap_or(true) {
        if let Ok(dm) = member.user.create_dm_channel(ctx).await {
            if dm
                .id
                .widen()
                .send_message(
                    ctx.http(),
                    serenity::CreateMessage::default().embed(dm_embed.clone()),
                )
                .await
                .is_ok()
            {
                dm_embed = dm_embed.field("User notified", "Yes", false);
            } else {
                dm_embed = dm_embed.field("User notified", "Failed", false);
            }
        } else {
            dm_embed = dm_embed.field("User notified", "Failed", false);
        }
    } else {
        dm_embed = dm_embed.field("User notified", "No", false);
    }

    if let Some(storage) = &ctx.data().storage {
        let guild_config = storage.get_config(member.guild_id).await?;

        if let Some(logs_channel) = guild_config.moderation_logs_channel {
            let server_embed = dm_embed.footer(
                serenity::CreateEmbedFooter::new(ctx.author().tag()).icon_url(ctx.author().face()),
            );

            logs_channel
                .send_message(
                    ctx.http(),
                    serenity::CreateMessage::default().embed(server_embed),
                )
                .await?;
        }
    }

    ctx.say("Success!").await?;

    Ok(())
}
