// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use regex::Regex;
use std::{collections::HashMap, sync::LazyLock, time::Duration};

use poise::{CreateReply, serenity_prelude as serenity};
use serde::{Deserialize, Serialize};

use crate::{Context, config::CONFIG, http::HTTP};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LighthouseAuditData {
    id: String,
    title: String,
    description: Option<String>,
    score: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LighthouseResultData {
    categories: HashMap<String, LighthouseAuditData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PagespeedResponse {
    lighthouse_result: LighthouseResultData,
}

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^https?:\/\/[-a-zA-Z0-9@:%._\+~#=]+\.[a-zA-Z0-9()]+\b[-a-zA-Z0-9()@:%_\+.~#?&//=]*$",
    )
    .unwrap()
});

/// Run Lighthouse on a URL using Google's PageSpeed API
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn lighthouse(
    ctx: Context<'_>,
    #[description = "The URL to test"] url: String,
) -> Result<()> {
    ctx.defer().await?;

    if !URL_REGEX.is_match(&url) {
        ctx.say("Invalid URL provided!").await?;
        return Ok(());
    }

    if let Some(key) = &CONFIG.pagespeed_api_key {
        let reply_handle = ctx
            .send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::new()
                        .title("Lighthouse audit in progress")
                        .description("This could take around a minute.")
                        .color(0x66d9e8)
                        .timestamp(serenity::Timestamp::now()),
                ),
            )
            .await?;

        let data: PagespeedResponse = HTTP
            .get("https://pagespeedonline.googleapis.com/pagespeedonline/v5/runPagespeed")
            .query(&[
                ("url", url.as_str()),
                ("strategy", "MOBILE"),
                ("category", "PERFORMANCE"),
                ("category", "ACCESSIBILITY"),
                ("category", "BEST_PRACTICES"),
                ("category", "SEO"),
                ("key", key.as_str()),
            ])
            .timeout(Duration::from_secs(90))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut report_embed = serenity::CreateEmbed::new()
            .title("Lighthouse report")
            .description(url)
            .color(0x74c0fc)
            .timestamp(serenity::Timestamp::now());

        for key in ["performance", "accessibility", "best-practices", "seo"] {
            if let Some(value) = data.lighthouse_result.categories.get(key) {
                report_embed =
                    report_embed.field(&value.title, format!("{:.0}", value.score * 100.0), false);
            }
        }

        reply_handle
            .edit(ctx, CreateReply::default().embed(report_embed))
            .await?;
    } else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("PageSpeed API not configured!")
                    .description(
                        "Contact the owner of this app if this command is supposed to be working.",
                    )
                    .color(0xff6b6b),
            ),
        )
        .await?;
    }

    Ok(())
}
