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
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[serenity::CreateComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(
                            r"### Translation unavailable!
There is no content to translate.",
                        ),
                    )])
                    .accent_color(0xffd43b),
                )]),
        )
        .await?;

        return Ok(());
    }

    if let Some(key) = &CONFIG.translation_api_key {
        let resp = translate_call(&message.content, key).await?;

        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            format!("### Translation\n{}", resp.translated_text),
                        )),
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            format!("-# *{}* → en", resp.detected_source_language),
                        )),
                    ])
                    .accent_color(0x34d399),
                )]),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[serenity::CreateComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(
                            r"### Cloud Translation API not configured!
Contact the owner of this app if this command is supposed to be working.",
                        ),
                    )])
                    .accent_color(0xff6b6b),
                )]),
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
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[serenity::CreateComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(
                            r"### Translation unavailable!
There is no content to translate.",
                        ),
                    )])
                    .accent_color(0xffd43b),
                )]),
        )
        .await?;

        return Ok(());
    }

    if let Some(key) = &CONFIG.translation_api_key {
        let resp = translate_call(&message.content, key).await?;

        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            format!("### Translation\n{}", resp.translated_text),
                        )),
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            format!("-# *{}* → en", resp.detected_source_language),
                        )),
                    ])
                    .accent_color(0x34d399),
                )]),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[serenity::CreateComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(
                            r"### Cloud Translation API not configured!
Contact the owner of this app if this command is supposed to be working.",
                        ),
                    )])
                    .accent_color(0xff6b6b),
                )]),
        )
        .await?;
    }

    Ok(())
}
