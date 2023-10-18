use crate::Context;
use anyhow::Result;
use poise::{serenity_prelude as serenity, CreateReply};

#[derive(serde::Deserialize)]
struct SafebooruResponse {
    file_url: reqwest::Url,
}

/// Fetch a random shiggy
#[poise::command(slash_command)]
pub async fn shiggy(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let mut url = "https://safebooru.donmai.us/posts/random.json".parse::<reqwest::Url>()?;
    url.query_pairs_mut()
        .append_pair("tags", "kemomimi-chan_(naga_u) naga_u")
        .append_pair("only", "file_url");

    let resp = crate::reqwest_client::HTTP.get(url).send().await?;

    if resp.status().is_success() {
        let data: SafebooruResponse = resp.json().await?;
        ctx.say(data.file_url).await?;
    } else {
        ctx.send(
            CreateReply::new().embed(
                serenity::CreateEmbed::new()
                    .title("Could not fetch shiggy!")
                    .description("An error occurred while fetching from the API.")
                    .color(0xef4444),
            ),
        )
        .await?;
    }

    Ok(())
}
