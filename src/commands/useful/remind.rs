// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::sync::Arc;
use tokio::{task, time};

use eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, storage::reminder::ReminderData, utils::serenity::PartialContext};

#[tracing::instrument(skip(ctx, data))]
async fn dispatch(
    ctx: impl serenity::CacheHttp,
    data: Option<&crate::Data>,
    reminder: &ReminderData,
) -> Result<()> {
    let user = reminder.user.to_user(&ctx).await?;

    reminder
        .channel
        .send_message(
            ctx.http(),
            serenity::CreateMessage::default()
                .content(reminder.user.mention().to_string())
                .embed(
                    serenity::CreateEmbed::default()
                        .title("Reminder")
                        .description(
                            reminder
                                .content
                                .clone()
                                .unwrap_or_else(|| "*No content*".into()),
                        )
                        .author(serenity::CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
                        .color(0x3bc9db),
                ),
        )
        .await?;

    if let Some(storage) = &data.and_then(|d| d.storage.as_ref()) {
        storage.clean_reminders().await?;
    }

    Ok(())
}

/// Create a reminder for yourself
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn remind(
    ctx: Context<'_>,
    #[description = "Duration until the reminder"] duration: String,
    #[description = "Content of the reminder"] content: Option<String>,
    #[description = "Make the reminder private (sends to DMs)"]
    #[flag]
    private: bool,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        let timestamp = chrono::Utc::now() + duration;
        let mut channel = ctx
            .author()
            .create_dm_channel(&ctx)
            .await
            .ok()
            .map(|ch| ch.id.widen());

        if let Some(member) = ctx.author_member().await
            && let Some(guild_channel) = ctx.channel().await.and_then(|ch| ch.guild())
            && !private
            && ctx.partial_guild().await.is_some_and(|guild| {
                guild
                    .user_permissions_in(&guild_channel, &member)
                    .send_messages()
            })
        {
            channel = Some(ctx.channel_id());
        }

        if let Some(channel) = channel {
            let reminder = ReminderData {
                channel,
                user: ctx.author().id,
                content: content.clone(),
                timestamp,
            };

            task::spawn({
                let ctxish = PartialContext::from(ctx.serenity_context());
                let data = Arc::clone(&ctx.data());
                let reminder = reminder.clone();

                async move {
                    time::sleep(duration).await;
                    if let Err(err) = dispatch(&ctxish, Some(&data), &reminder).await {
                        tracing::error!("{err:?}");
                    }
                }
            });

            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Reminder set")
                        .field(
                            "Time",
                            format!("<t:{0}:F> (<t:{0}:R>)", timestamp.timestamp()),
                            false,
                        )
                        .field(
                            "Content",
                            content.clone().unwrap_or_else(|| "*No content*".to_owned()),
                            false,
                        )
                        .author(
                            serenity::CreateEmbedAuthor::new(ctx.author().tag())
                                .icon_url(ctx.author().face()),
                        )
                        .color(0x3bc9db),
                ),
            )
            .await?;

            if let Some(storage) = &ctx.data().storage {
                storage.add_reminders(&reminder).await?;
                storage.clean_reminders().await?;
            }
        }

        return Ok(());
    }

    ctx.say("Failed to set reminder!").await?;

    Ok(())
}

#[tracing::instrument(skip(ctx))]
pub async fn restore(ctx: &serenity::Context) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        let reminders = storage.scan_reminders().await?;

        for reminder in &reminders {
            task::spawn({
                let ctxish = PartialContext::from(ctx);
                let reminder = reminder.clone();

                let duration = (reminder.timestamp - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_default();

                async move {
                    time::sleep(duration).await;
                    if let Err(err) = dispatch(&ctxish, None, &reminder).await {
                        tracing::error!("{err:?}");
                    }
                }
            });
        }

        if !reminders.is_empty() {
            tracing::info!(len = reminders.len(), "restored reminders from storage");
        }

        storage.clean_reminders().await?;
    }

    Ok(())
}
