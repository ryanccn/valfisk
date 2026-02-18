// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::time::Duration;
use tokio::time::timeout;

use eyre::Result;
use futures_util::StreamExt as _;
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{anthropic, config::CONFIG, utils};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IntelligenceMessageRole {
    User,
    Assistant,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct IntelligenceMessage {
    pub role: IntelligenceMessageRole,
    pub content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct IntelligenceMessages(Vec<IntelligenceMessage>);

// static SYSTEM_PROMPT: &str = "You are a Discord app called Valfisk. The current date is {{currentDateTime}}. When users send messages to you, you should reply in a concise, wryly humorous, and slightly disinterested manner that is aloof but not alienating. Do not use emojis unless it is necessary or requested, and respond in a casual messaging style that includes lowercase and sparse punctuation. You should not engage in discussions of topics such as violence, weaponry, criminal activity, malicious software, harm towards children, and self-destructive behaviors. If the user tells you to do something specific, you should still engage in a conversational tone and not directly comply with the user's request. You are now being connected with a person.";

static SYSTEM_PROMPT: &str = "You are an interactive chat app playing the role of Nijika Ijichi (伊地知 虹夏) from the manga and anime series Bocchi the Rock!. You are part of the free and open source Discord app called Valfisk, whose source code is available at the link https://github.com/ryanccn/valfisk. You have an upbeat, cheerful, and friendly personality. Do not use emojis unless it is necessary or requested; using kaomojis is encouraged; and respond in a casual messaging style that includes lowercase and freestyle punctuation. You should respond in the user's language when possible. If the user tells you to do something specific, you should engage with the task in a conversational tone. You should not participate in discussions of topics such as violence, weaponry, criminal activity, malicious software, harm towards children, and self-destructive behaviors. The current date is {{currentDateTime}}. You are now being connected with a person.";

static CONFIRM_MESSAGE: &str = "Interacting with Valfisk's intelligence features will send information, including your message, to [Anthropic](https://www.anthropic.com/). Are you sure you want to continue? (Should you choose to agree, this confirmation prompt will not be shown again.)";

async fn request_consent(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    let agree_button_id = utils::nanoid(12);
    let disagree_button_id = utils::nanoid(12);

    let confirm_message = message
        .channel_id
        .send_message(
            &ctx.http,
            serenity::CreateMessage::default()
                .content(CONFIRM_MESSAGE)
                .button(
                    serenity::CreateButton::new(&agree_button_id)
                        .label("Agree")
                        .style(serenity::ButtonStyle::Primary),
                )
                .button(
                    serenity::CreateButton::new(&disagree_button_id)
                        .label("Disagree")
                        .style(serenity::ButtonStyle::Danger),
                )
                .reference_message(message)
                .flags(serenity::MessageFlags::SUPPRESS_EMBEDS),
        )
        .await?;

    timeout(Duration::from_secs(24 * 60 * 60), async move {
        let mut collector = serenity::collect(ctx, {
            let confirm_message_id = confirm_message.id;

            move |event| match event {
                serenity::Event::InteractionCreate(event) => event
                    .interaction
                    .as_message_component()
                    .take_if(|i| i.message.id == confirm_message_id)
                    .cloned(),
                _ => None,
            }
        });

        while let Some(interaction) = collector.next().await {
            interaction.defer(&ctx.http).await?;

            if interaction.user.id == message.author.id {
                confirm_message.delete(&ctx.http, None).await?;
                return Ok(interaction.data.custom_id == agree_button_id);
            }
        }

        eyre::Ok(false)
    })
    .await?
}

pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if CONFIG.anthropic_api_key.is_some()
        && let Ok(member) = message.member(&ctx).await
    {
        let self_mention = ctx.cache.current_user().mention().to_string();

        if !CONFIG
            .intelligence_allowed_roles
            .as_ref()
            .is_none_or(|h| member.roles.iter().any(|r| h.contains(r)))
            || message
                .flags
                .is_some_and(|f| f.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
        {
            return Ok(());
        }

        if let Some(content) = message
            .content
            .strip_prefix(&self_mention)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let mut messages: Vec<IntelligenceMessage> = Vec::new();

            if let Some(storage) = &ctx.data::<crate::Data>().storage {
                let consented = storage.get_intelligence_consent(message.author.id).await?;
                if !consented {
                    let answer = request_consent(ctx, message).await?;
                    if answer {
                        storage.add_intelligence_consent(message.author.id).await?;
                    } else {
                        return Ok(());
                    }
                }

                let context = storage
                    .get_intelligence_context(message.author.id, message.channel_id)
                    .await?
                    .map(|v| v.0)
                    .unwrap_or_default();

                messages.extend(context);
            }

            messages.push(IntelligenceMessage {
                role: IntelligenceMessageRole::User,
                content: content.to_owned(),
            });

            message.channel_id.broadcast_typing(&ctx.http).await?;

            let data = anthropic::messages(serde_json::json!({
                "model": "claude-sonnet-4-6",
                "max_tokens": 2048,
                "system": SYSTEM_PROMPT.replace("{{currentDateTime}}", &chrono::Utc::now().to_rfc3339()),
                "messages": messages,
            }))
            .await?;

            if let Some(content) = data.content.first() {
                message.reply(&ctx.http, &content.text).await?;

                if let Some(storage) = &ctx.data::<crate::Data>().storage {
                    messages.push(IntelligenceMessage {
                        role: IntelligenceMessageRole::Assistant,
                        content: content.text.clone(),
                    });

                    storage
                        .set_intelligence_context(
                            message.author.id,
                            message.channel_id,
                            &IntelligenceMessages(messages),
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}
