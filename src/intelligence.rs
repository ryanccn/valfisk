// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env;

use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use crate::reqwest_client::HTTP;

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
    let secret = env::var("INTELLIGENCE_SECRET")
        .map_err(|_| eyre!("Valfisk Intelligence API secret is not set!"))?;

    let resp: Response = HTTP
        .post(API_URL)
        .bearer_auth(secret)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.response)
}
