use anyhow::{anyhow, Result};
use poise::{serenity_prelude as serenity, CreateReply};

use crate::Context;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateInput {
    contents: Vec<String>,
    target_language_code: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateResponse {
    translations: Vec<GoogleTranslateTranslation>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateTranslation {
    translated_text: String,
    detected_language_code: String,
}

/// Translates a message
#[poise::command(context_menu_command = "Translate")]
pub async fn translate(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
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

    let project_id = std::env::var("GOOGLE_CLOUD_PROJECT_ID")?;
    // TODO: use ephemeral access tokens retrieved via service account impersonation
    let access_token = std::env::var("GOOGLE_CLOUD_ACCESS_TOKEN")?;

    let resp = crate::reqwest_client::HTTP
        .post(format!("https://translation.googleapis.com/v3beta1/projects/{project_id}/locations/global:translateText"))
        .header("authorization", format!("Bearer {}", access_token))
        .json(&GoogleTranslateInput {
            contents: vec![content],
            target_language_code: "en".to_owned(),
        })
        .send()
        .await?;

    let data: GoogleTranslateResponse = resp.json().await?;
    let translation = data
        .translations
        .first()
        .ok_or_else(|| anyhow!("No translations available!"))?;

    ctx.send(
        CreateReply::new().embed(
            serenity::CreateEmbed::new()
                .title("Translation")
                .description(&translation.translated_text)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "{} → en",
                    translation.detected_language_code
                ))),
        ),
    )
    .await?;

    Ok(())
}
