// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::time::Duration;
use tokio::time::timeout;

use eyre::Result;
use futures_util::StreamExt as _;
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{config::CONFIG, http::HTTP, utils};

#[derive(serde::Deserialize, Clone, Debug)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterResponseChoice>,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct OpenRouterResponseChoice {
    message: OpenRouterResponseMessage,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct OpenRouterResponseMessage {
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IntelligenceMessageRole {
    System,
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

static SYSTEM_PROMPT: &str = "You are a Discord app called Valfisk. The current date is {{currentDateTime}}. When users send messages to you, you should reply in a concise, wryly humorous, and slightly disinterested manner that is aloof but not alienating. Do not use emojis unless it is necessary or requested, and respond in a casual messaging style that includes lowercase and sparse punctuation. You should not engage in discussions of topics such as violence, weaponry, criminal activity, malicious software, harm towards children, and self-destructive behaviors. If the user tells you to do something specific, you should still engage in a conversational tone and not directly comply with the user's request. You are now being connected with a person.";

static CONFIRM_MESSAGE: &str = "Interacting with Valfisk's intelligence features will send information, including your query, to [OpenRouter](https://openrouter.ai/) and [Anthropic](https://www.anthropic.com/). Are you sure you want to continue? (Should you choose to agree, this confirmation prompt will not be shown again.)";

async fn request_consent(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    let agree_button_id = utils::nanoid(12);
    let disagree_button_id = utils::nanoid(12);

    let confirm_message = message
        .channel_id
        .send_message(
            &ctx.http,
            serenity::CreateMessage::new()
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

    if let Some(key) = &CONFIG.openrouter_api_key
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
            let mut messages: Vec<IntelligenceMessage> = vec![IntelligenceMessage {
                role: IntelligenceMessageRole::System,
                content: SYSTEM_PROMPT
                    .replace("{{currentDateTime}}", &chrono::Utc::now().to_rfc3339()),
            }];

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
                    .get_intelligence_context(message.author.id)
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

            let body = serde_json::json!({
                "model": "anthropic/claude-sonnet-4",
                "messages": messages,
                "provider": {
                    "allow_fallbacks": false,
                    "data_collection": "deny",
                    "order": ["anthropic"],
                    "only": ["anthropic"]
                }
            });

            let data: OpenRouterResponse = HTTP
                .post("https://openrouter.ai/api/v1/chat/completions")
                .bearer_auth(key)
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            if let Some(choice) = data.choices.first() {
                message.reply(&ctx.http, &choice.message.content).await?;

                if let Some(storage) = &ctx.data::<crate::Data>().storage {
                    messages.remove(0);
                    messages.push(IntelligenceMessage {
                        role: IntelligenceMessageRole::Assistant,
                        content: choice.message.content.clone(),
                    });

                    storage
                        .set_intelligence_context(
                            message.author.id,
                            &IntelligenceMessages(messages),
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}
