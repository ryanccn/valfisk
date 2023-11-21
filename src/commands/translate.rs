use anyhow::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::Context;

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateSentence {
    trans: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateResponse {
    src: String,
    sentences: Vec<GoogleTranslateSentence>,
}

/// Translates a message
#[poise::command(context_menu_command = "Translate", guild_only)]
pub async fn translate(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    ctx.defer().await?;
    let serenity::Message { content, .. } = message;

    if content.is_empty() {
        ctx.send(
            CreateReply::new().embed(
                serenity::CreateEmbed::new()
                    .title("Translation unavailable")
                    .description("There is no content to translate")
                    .color(0xfacc15),
            ),
        )
        .await?;

        return Ok(());
    }

    let mut api_url =
        "https://translate.googleapis.com/translate_a/single".parse::<reqwest::Url>()?;

    api_url
        .query_pairs_mut()
        .append_pair("client", "gtx")
        .append_pair("sl", "auto")
        .append_pair("tl", "en")
        .append_pair("dt", "t")
        .append_pair("dj", "1")
        .append_pair("source", "input")
        .append_pair("q", &content);

    let resp = crate::reqwest_client::HTTP.get(api_url).send().await?;

    let data: GoogleTranslateResponse = resp.json().await?;
    let translation = data
        .sentences
        .into_iter()
        .filter_map(|s| s.trans)
        .collect::<String>();

    ctx.send(
        CreateReply::new().embed(
            serenity::CreateEmbed::new()
                .title("Translation")
                .description(&translation)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "{} → en",
                    data.src
                ))),
        ),
    )
    .await?;

    Ok(())
}
