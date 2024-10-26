// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use crate::{config::CONFIG, reqwest_client::HTTP};

static API_URL: &str = "https://intelligence.valfisk.ryanccn.dev/v2";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMetadata {
    pub username: String,
    pub display_name: Option<String>,
    pub nick: Option<String>,
}

#[derive(Serialize)]
pub struct Request {
    pub query: String,
    pub metadata: RequestMetadata,
}

#[derive(Deserialize)]
pub struct Response {
    pub response: String,
}

pub async fn query(request: Request) -> Result<String> {
    let resp: Response = HTTP
        .post(API_URL)
        .bearer_auth(
            CONFIG
                .intelligence_secret
                .as_ref()
                .ok_or_else(|| eyre!("could not obtain intelligence API secret"))?,
        )
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.response)
}
