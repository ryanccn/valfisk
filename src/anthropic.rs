// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};

use crate::{config::CONFIG, http::HTTP};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AnthropicContent {
    pub text: String,
}

pub async fn messages(body: impl serde::Serialize) -> Result<AnthropicResponse> {
    let data: AnthropicResponse = HTTP
        .post("https://api.anthropic.com/v1/messages")
        .header("anthropic-version", "2023-06-01")
        .header(
            "x-api-key",
            CONFIG
                .anthropic_api_key
                .as_ref()
                .ok_or_else(|| eyre!("ANTHROPIC_API_KEY unavailable"))?,
        )
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(data)
}
