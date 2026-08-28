// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{analytics, utils};

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(false);
    }

    if let Some(guild_id) = message.guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage
        && let Ok(config) = storage.get_config(guild_id).await
        && let Some(honeypot_channel) = config.honeypot_channel
        && honeypot_channel == message.channel_id
    {
        message.delete(&ctx.http, Some("Honeypot")).await?;

        let timed_out = if let Ok(mut member) = message.member(&ctx).await {
            member
                .disable_communication_until(
                    &ctx.http,
                    (chrono::Utc::now() + chrono::TimeDelta::days(1)).into(),
                )
                .await
                .is_ok()
        } else {
            false
        };

        if let Some(logs_channel) = config.message_logs_channel {
            let mut components = vec![];

            if let Some(role) = config.moderator_role {
                components.push(serenity::CreateComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(role.mention().to_string()),
                ));
            }

            components.push(serenity::CreateComponent::Container(
                serenity::CreateContainer::new(vec![
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new("### Honeypot"),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Author**\n{} (*{}*)",
                            utils::serenity::format_mentionable(Some(message.author.id)),
                            if timed_out {
                                "timed out"
                            } else {
                                "timeout failed"
                            }
                        )),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Channel**\n{}",
                            utils::serenity::format_mentionable(Some(message.channel_id))
                        )),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Content**\n{}",
                            utils::truncate(&message.content, 1024)
                        )),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Attachments**\n{}",
                            utils::serenity::format_attachments(&message.attachments),
                        )),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "-# {}",
                            serenity::FormattedTimestamp::now()
                        )),
                    ),
                ])
                .accent_color(0xff6b6b),
            ));

            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(
                            serenity::CreateAllowedMentions::new()
                                .roles(config.moderator_role.iter().copied().collect::<Vec<_>>()),
                        )
                        .components(&components),
                )
                .await?;
        }

        analytics::send_honeypot(message.guild_id).await;

        return Ok(true);
    }

    Ok(false)
}
