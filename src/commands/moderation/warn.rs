// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, utils};

/// Warn a member
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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
    #[description = "The member to warn"] member: serenity::Member,
    #[description = "Reason for the warn"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let partial_guild = ctx
        .partial_guild()
        .await
        .ok_or_else(|| eyre!("failed to obtain partial guild"))?;

    let mut container =
        serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "### Warn\n{}",
                utils::serenity::format_mentionable(Some(member.user.id)),
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

        if let Ok(dm) = member.user.create_dm_channel(ctx).await
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

    if let Some(storage) = &ctx.data().storage {
        let guild_config = storage.get_config(member.guild_id).await?;

        if let Some(logs_channel) = guild_config.moderation_logs_channel {
            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
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
                        .components(&[serenity::CreateComponent::Container(container)]),
                )
                .await?;
        }
    }

    ctx.say("Success!").await?;

    Ok(())
}
