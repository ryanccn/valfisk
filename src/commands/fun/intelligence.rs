use std::env;

use color_eyre::eyre::{eyre, Context as _, Result};
use serde::Deserialize;
use serde_json::json;

use crate::{reqwest_client::HTTP, Context};

static ASK_API_URL: &str = "https://intelligence.valfisk.ryanccn.dev/ask";

#[derive(Deserialize)]
struct AskResponse {
    response: String,
}

/// Ask Valfisk Intelligence™
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "The query to ask about"] query: String,
) -> Result<()> {
    ctx.defer().await?;

    let secret = env::var("INTELLIGENCE_SECRET")
        .wrap_err_with(|| eyre!("Valfisk Intelligence API secret is not set!"))?;

    let resp: AskResponse = HTTP
        .post(ASK_API_URL)
        .bearer_auth(secret)
        .json(&json!({ "query": query }))
        .send()
        .await?
        .json()
        .await?;

    ctx.say(&resp.response).await?;

    Ok(())
}
