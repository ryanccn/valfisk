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
            .check_urls(
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

            if let Some(guild_id) = message.guild_id {
                if let Some(storage) = &ctx.data::<crate::Data>().storage {
                    let guild_config = storage.get_config(guild_id).await?;

                    if let Some(logs_channel) = guild_config.message_logs_channel {
                        let embed = serenity::CreateEmbed::default()
                            .title("Safe Browsing")
                            .field(
                                "Channel",
                                utils::serenity::format_mentionable(Some(message.channel_id)),
                                false,
                            )
                            .field(
                                "Author",
                                utils::serenity::format_mentionable(Some(message.author.id)),
                                false,
                            )
                            .field(
                                "Author timed out",
                                if timed_out { "Yes" } else { "Failed" },
                                false,
                            )
                            .field("Content", utils::truncate(&content, 1024), false)
                            .field(
                                "URLs",
                                matches
                                    .iter()
                                    .map(|m| format!("`{}` → {}", m.0, m.1.threat_type))
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                                false,
                            )
                            .color(0xff6b6b)
                            .timestamp(serenity::Timestamp::now());

                        logs_channel
                            .send_message(
                                &ctx.http,
                                serenity::CreateMessage::default()
                                    .content(
                                        guild_config
                                            .moderator_role
                                            .map(|r| r.mention().to_string())
                                            .unwrap_or_default(),
                                    )
                                    .embed(embed),
                            )
                            .await?;
                    }
                }
            }

            return Ok(true);
        }
    }

    Ok(false)
}
