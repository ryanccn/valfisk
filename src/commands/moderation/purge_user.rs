// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, utils};

/// Kick a user and purge recent messages from them
#[tracing::instrument(skip(ctx, user), fields(user = user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    rename = "purge-user",
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn purge_user(
    ctx: Context<'_>,
    #[description = "The user to purge"] user: serenity::User,
    #[description = "Reason for the purge"] reason: Option<String>,

    #[description = "Days of messages to delete (default: 1)"]
    #[min = 0]
    #[max = 7]
    delete_message_days: Option<u32>,

    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    let delete_message_days = delete_message_days.unwrap_or(1);

    ctx.defer_ephemeral().await?;

    let partial_guild = ctx
        .partial_guild()
        .await
        .ok_or_else(|| eyre!("failed to obtain partial guild"))?;

    let mut container =
        serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "### Purge\n{}",
                utils::serenity::format_mentionable(Some(user.id)),
            )),
        )])
        .accent_color(0xda77f2);

    if let Some(reason) = &reason {
        container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!("**Reason**\n{reason}")),
        ));
    }

    if dm.unwrap_or(true) {
        let dm_container =
            container
                .clone()
                .add_component(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "-# {} \u{00B7} {}",
                        partial_guild.name,
                        serenity::FormattedTimestamp::now()
                    )),
                ));

        if let Ok(dm) = user.create_dm_channel(ctx).await
            && dm
                .id
                .widen()
                .send_message(
                    ctx.http(),
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(vec![serenity::CreateComponent::Container(dm_container)]),
                )
                .await
                .is_ok()
        {
            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new("**User notified**\nYes"),
            ));
        } else {
            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new("**User notified**\nFailed"),
            ));
        }
    } else {
        container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new("**User notified**\nNo"),
        ));
    }

    container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "**Days of messages deleted**\n{delete_message_days}"
        )),
    ));

    let reply_container = container.clone();

    if let Some(storage) = &ctx.data().storage {
        let guild_config = storage.get_config(partial_guild.id).await?;

        if let Some(logs_channel) = guild_config.moderation_logs_channel {
            let log_container =
                container.add_component(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "-# {} \u{00B7} {}",
                        ctx.author().mention(),
                        serenity::FormattedTimestamp::now()
                    )),
                ));

            logs_channel
                .send_message(
                    ctx.http(),
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(vec![serenity::CreateComponent::Container(log_container)]),
                )
                .await?;
        }
    }

    partial_guild
        .id
        .ban(
            ctx.http(),
            user.id,
            delete_message_days * 86400,
            reason.as_deref(),
        )
        .await?;

    partial_guild.id.unban(ctx.http(), user.id, None).await?;

    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(vec![serenity::CreateComponent::Container(reply_container)]),
    )
    .await?;

    Ok(())
}
