// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use std::{collections::HashMap, env, time::Duration};

use poise::{serenity_prelude as serenity, CreateReply};
use serde::{Deserialize, Serialize};

use crate::{reqwest_client::HTTP, Context};

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

/// Run Lighthouse on a URL using Google's PageSpeed API
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn lighthouse(
    ctx: Context<'_>,
    #[description = "The URL to test"] url: String,
) -> Result<()> {
    ctx.defer().await?;

    if let Ok(pagespeed_token) = env::var("PAGESPEED_API_KEY") {
        let reply_handle = ctx
            .send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::new()
                        .title("Lighthouse audit in progress")
                        .description("This could take around a minute!")
                        .color(0x66d9e8)
                        .timestamp(serenity::Timestamp::now()),
                ),
            )
            .await?;

        let mut api_url = reqwest::Url::parse(
            "https://pagespeedonline.googleapis.com/pagespeedonline/v5/runPagespeed",
        )?;

        api_url
            .query_pairs_mut()
            .append_pair("url", &url)
            .append_pair("strategy", "MOBILE")
            .append_pair("category", "PERFORMANCE")
            .append_pair("category", "ACCESSIBILITY")
            .append_pair("category", "BEST_PRACTICES")
            .append_pair("category", "SEO")
            .append_pair("key", &pagespeed_token);

        let resp = HTTP
            .get(api_url)
            .timeout(Duration::from_secs(60))
            .send()
            .await?
            .error_for_status()?;

        let data: PagespeedResponse = resp.json().await?;

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
                    .title(r"PageSpeed API key not provided!")
                    .description(r"The `PAGESPEED_API_KEY` environment variable is required to be set to use this command."),
            ),
        )
        .await?;
    }

    Ok(())
}
