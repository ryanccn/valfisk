// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use eyre::Result;
use serde_json::json;

use crate::{Context, config::CONFIG, http::HTTP};

#[tracing::instrument(skip(data))]
pub async fn send(name: &str, data: impl serde::Serialize) -> Result<()> {
    if let Some(endpoint) = &CONFIG.umami_endpoint
        && let Some(website_id) = &CONFIG.umami_website_id
        && let Some(hostname) = &CONFIG.umami_hostname
    {
        HTTP.post(endpoint)
            .json(&json!({
              "payload": {
                "website": website_id,
                "hostname": hostname,
                "url": "/",
                "language": "en-US",
                "name": name,
                "data": data
              },
              "type": "event"
            }))
            .send()
            .await?
            .error_for_status()?;
    }

    Ok(())
}

pub async fn send_command(ctx: Context<'_>) {
    if ctx.command().owners_only {
        return;
    }

    if let Err(err) = send(
        "command_v1",
        json!({ "name": ctx.command().name, "guild": ctx.guild_id() }),
    )
    .await
    {
        tracing::warn!("{err:?}");
    }
}

pub async fn send_message(guild: Option<serenity::GuildId>) {
    if let Err(err) = send("message_v1", json!({ "guild": guild })).await {
        tracing::warn!("{err:?}");
    }
}

pub async fn send_code_expansion(guild: Option<serenity::GuildId>) {
    if let Err(err) = send("code_expansion_v1", json!({ "guild": guild })).await {
        tracing::warn!("{err:?}");
    }
}

pub async fn send_safe_browsing(guild: Option<serenity::GuildId>) {
    if let Err(err) = send("safe_browsing_v1", json!({ "guild": guild })).await {
        tracing::warn!("{err:?}");
    }
}
