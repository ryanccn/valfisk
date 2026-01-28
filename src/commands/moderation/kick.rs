// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

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

    let partial_guild = ctx
        .partial_guild()
        .await
        .ok_or_else(|| eyre!("failed to obtain partial guild"))?;

    let extra_message = if let Some(storage) = &ctx.data().storage {
        let guild_config = storage.get_config(partial_guild.id).await?;
        guild_config.moderation_extra_message_kick
    } else {
        None
    };

    let user_reason = utils::option_strings(reason.as_deref(), extra_message.as_deref());

    member.kick(ctx.http(), reason.as_deref()).await?;

    let mut container =
        serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "### Kick\n{}",
                utils::serenity::format_mentionable(Some(member.user.id)),
            )),
        )])
        .accent_color(0xf783ac);

    if let Some(user_reason) = &user_reason {
        container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!("**Reason**\n{user_reason}")),
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
