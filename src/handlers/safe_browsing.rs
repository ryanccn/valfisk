// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{self as serenity, Mentionable as _};
use regex::Regex;
use std::sync::LazyLock;

use eyre::Result;

use crate::utils;

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?:\/\/[-a-zA-Z0-9@:%._\+~#=]+\.[a-zA-Z0-9()]+\b[-a-zA-Z0-9()@:%_\+.~#?&//=]*")
        .unwrap()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(false);
    }

    if let Some(safe_browsing) = &ctx.data::<crate::Data>().safe_browsing {
        let content = message.content.to_string();

        let matches = safe_browsing
            .check(
                &URL_REGEX
                    .find_iter(&content)
                    .map(|u| u.as_str())
                    .collect::<Vec<_>>(),
            )
            .await?;

        if !matches.is_empty() {
            message
                .delete(&ctx.http, Some("URL(s) flagged by Safe Browsing"))
                .await?;

            let timed_out = if let Ok(mut member) = message.member(&ctx).await {
                member
                    .disable_communication_until(
                        &ctx.http,
                        (chrono::Utc::now() + chrono::TimeDelta::hours(1)).into(),
                    )
                    .await
                    .is_ok()
            } else {
                false
            };

            if let Some(guild_id) = message.guild_id
                && let Some(storage) = &ctx.data::<crate::Data>().storage
            {
                let guild_config = storage.get_config(guild_id).await?;

                if let Some(logs_channel) = guild_config.message_logs_channel {
                    let mut components = vec![];

                    if let Some(role) = guild_config.moderator_role {
                        components.push(serenity::CreateComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(role.mention().to_string()),
                        ));
                    }

                    components.push(serenity::CreateComponent::Container(
                        serenity::CreateContainer::new(vec![
                            serenity::CreateContainerComponent::TextDisplay(
                                serenity::CreateTextDisplay::new(format!(
                                    "### Safe Browsing\n{}",
                                    matches
                                        .iter()
                                        .map(|m| format!("`{}` → {}", m.0, m.1.threat_type))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                )),
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
                                    serenity::CreateAllowedMentions::new().roles(
                                        guild_config
                                            .moderator_role
                                            .iter()
                                            .copied()
                                            .collect::<Vec<_>>(),
                                    ),
                                )
                                .components(&components),
                        )
                        .await?;
                }
            }

            return Ok(true);
        }
    }

    Ok(false)
}
