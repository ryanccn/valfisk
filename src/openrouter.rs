use eyre::{Result, eyre};

use crate::{config::CONFIG, http::HTTP};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenRouterResponse {
    pub choices: Vec<OpenRouterResponseChoice>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenRouterResponseChoice {
    pub message: OpenRouterResponseMessage,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenRouterResponseMessage {
    pub content: String,
}

pub async fn chat(body: impl serde::Serialize) -> Result<OpenRouterResponse> {
    let data: OpenRouterResponse = HTTP
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(
            CONFIG
                .openrouter_api_key
                .as_ref()
                .ok_or_else(|| eyre!("OPENROUTER_API_KEY unavailable"))?,
        )
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(data)
}
