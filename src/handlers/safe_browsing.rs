// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{self as serenity, Mentionable as _};
use regex::Regex;
use std::sync::LazyLock;

use eyre::Result;

use super::log::format_user;
use crate::{config::CONFIG, utils, Data};

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)",
    )
    .unwrap()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(
    ctx: &serenity::Context,
    data: &Data,
    message: &serenity::Message,
) -> Result<bool> {
    if message.guild_id != CONFIG.guild_id {
        return Ok(false);
    }

    if message.author.id == ctx.cache.current_user().id {
        return Ok(false);
    }

    if let Some(safe_browsing) = &data.safe_browsing {
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

            if let Ok(mut member) = message.member(&ctx).await {
                member
                    .disable_communication_until(
                        &ctx.http,
                        (chrono::Utc::now() + chrono::TimeDelta::minutes(10)).into(),
                    )
                    .await?;
            }

            if let Some(logs_channel) = CONFIG.message_logs_channel {
                let embed = serenity::CreateEmbed::default()
                    .title("Safe Browsing")
                    .field("Channel", message.channel_id.mention().to_string(), false)
                    .field("Author", format_user(Some(&message.author.id)), false)
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
                                CONFIG
                                    .moderator_role
                                    .map(|r| r.mention().to_string())
                                    .unwrap_or_default(),
                            )
                            .embed(embed),
                    )
                    .await?;
            }

            return Ok(true);
        }
    }

    Ok(false)
}
