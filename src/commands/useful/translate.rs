// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::{CreateReply, serenity_prelude as serenity};

use crate::{Context, config::CONFIG, http::HTTP};

#[derive(serde::Deserialize, Debug)]
struct GoogleTranslateResponse {
    data: GoogleTranslateTranslations,
}

#[derive(serde::Deserialize, Debug)]
struct GoogleTranslateTranslations {
    translations: Vec<GoogleTranslateTranslation>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateTranslation {
    translated_text: String,
    detected_source_language: String,
}

async fn translate_call(src: &str) -> Result<GoogleTranslateTranslation> {
    let token = CONFIG
        .translation_api_key
        .as_deref()
        .ok_or_else(|| eyre!("could not obtain `translation_api_key` from environment"))?;

    let GoogleTranslateResponse { data } = HTTP
        .get("https://translation.googleapis.com/language/translate/v2")
        .query(&[
            ("q", src),
            ("target", "en"),
            ("format", "text"),
            ("key", token),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    data.translations
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("did not receive translation from Google Cloud Translation API"))
}

/// Translates a message
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    context_menu_command = "Translate",
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn translate(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    ctx.defer().await?;

    if message.content.is_empty() {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Translation unavailable")
                    .description("There is no content to translate")
                    .color(0xffd43b),
            ),
        )
        .await?;

        return Ok(());
    }

    let resp = translate_call(&message.content).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Translation")
                .description(&resp.translated_text)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "{} → en",
                    resp.detected_source_language
                ))),
        ),
    )
    .await?;

    Ok(())
}

/// Translates a message
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    context_menu_command = "Translate (ephemeral)",
    ephemeral,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn translate_ephemeral(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if message.content.is_empty() {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Translation unavailable")
                    .description("There is no content to translate")
                    .color(0xffd43b),
            ),
        )
        .await?;

        return Ok(());
    }

    let resp = translate_call(&message.content).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Translation")
                .description(&resp.translated_text)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "{} → en",
                    resp.detected_source_language
                ))),
        ),
    )
    .await?;

    Ok(())
}
