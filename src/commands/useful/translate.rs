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

async fn translate_call(src: &str, key: &str) -> Result<GoogleTranslateTranslation> {
    let GoogleTranslateResponse { data } = HTTP
        .get("https://translation.googleapis.com/language/translate/v2")
        .query(&[
            ("q", src),
            ("target", "en"),
            ("format", "text"),
            ("key", key),
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

    if let Some(key) = &CONFIG.translation_api_key {
        let resp = translate_call(&message.content, key).await?;

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
    } else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("Cloud Translation API not configured!")
                    .description(
                        "Contact the owner of this app if this command is supposed to be working.",
                    )
                    .color(0xff6b6b),
            ),
        )
        .await?;
    }

    Ok(())
}

/// Translates a message
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    context_menu_command = "Translate (ephemeral)",
    rename = "translate-ephemeral",
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

    if let Some(key) = &CONFIG.translation_api_key {
        let resp = translate_call(&message.content, key).await?;

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
    } else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("Cloud Translation API not configured!")
                    .description(
                        "Contact the owner of this app if this command is supposed to be working.",
                    )
                    .color(0xff6b6b),
            ),
        )
        .await?;
    }

    Ok(())
}
