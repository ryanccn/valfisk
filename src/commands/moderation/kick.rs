// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{Context, utils};

/// Kick a member
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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
    #[description = "The member to kick"] member: serenity::Member,
    #[description = "Reason for the kick"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    member.kick(ctx.http(), reason.as_deref()).await?;

    let mut dm_embed = serenity::CreateEmbed::default()
        .title("Kick")
        .field(
            "User",
            utils::serenity::format_mentionable(Some(member.user.id)),
            false,
        )
        .color(0xf783ac)
        .timestamp(serenity::Timestamp::now());

    if let Some(reason) = &reason {
        dm_embed = dm_embed.field("Reason", reason, false);
    }

    if dm.unwrap_or(true) {
        if let Ok(dm) = member.user.create_dm_channel(ctx.http()).await {
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
