// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, utils};

/// Ban a member
#[tracing::instrument(skip(ctx, member), fields(member = member.user.id.get(), ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
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

    let partial_guild = ctx
        .partial_guild()
        .await
        .ok_or_else(|| eyre!("failed to obtain partial guild"))?;

    let extra_message = if let Some(storage) = &ctx.data().storage {
        let guild_config = storage.get_config(partial_guild.id).await?;
        guild_config.moderation_extra_message_ban
    } else {
        None
    };

    let user_reason = utils::option_strings(reason.as_deref(), extra_message.as_deref());

    member
        .ban(
            ctx.http(),
            delete_message_days.map_or(0, |d| d * 86400),
            reason.as_deref(),
        )
        .await?;

    let mut container =
        serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "### Ban\n{}",
                utils::serenity::format_mentionable(Some(member.user.id)),
            )),
        )])
        .accent_color(0xda77f2);

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
            container = container
                .add_component(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "**Days of messages deleted**\n{}",
                        delete_message_days.unwrap_or(0)
                    )),
                ))
                .add_component(serenity::CreateContainerComponent::TextDisplay(
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
