// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::{reqwest_client::HTTP, Context};

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

#[derive(Debug)]
struct GoogleTranslateResult {
    translation: String,
    src: String,
}

async fn translate_call(src: &str) -> Result<GoogleTranslateResult> {
    let data: GoogleTranslateResponse = HTTP
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[
            ("client", "gtx"),
            ("sl", "auto"),
            ("tl", "en"),
            ("dt", "t"),
            ("dj", "1"),
            ("source", "input"),
            ("q", src),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let translation = data
        .sentences
        .into_iter()
        .filter_map(|s| s.trans)
        .collect::<String>();

    Ok(GoogleTranslateResult {
        translation,
        src: data.src,
    })
}

/// Translates a message
#[poise::command(context_menu_command = "Translate", guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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

    let GoogleTranslateResult { translation, src } = translate_call(&message.content).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Translation")
                .description(&translation)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!("{src} → en"))),
        ),
    )
    .await?;

    Ok(())
}

/// Translates a message
#[poise::command(context_menu_command = "Translate (private)", guild_only, ephemeral)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn translate_private(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
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

    let GoogleTranslateResult { translation, src } = translate_call(&message.content).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Translation")
                .description(&translation)
                .color(0x34d399)
                .footer(serenity::CreateEmbedFooter::new(format!("{src} → en"))),
        ),
    )
    .await?;

    Ok(())
}
