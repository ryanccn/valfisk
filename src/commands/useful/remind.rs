// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};
use tokio::{task, time};

use crate::{Context, storage::reminder::ReminderData};

#[tracing::instrument(skip(http))]
async fn dispatch(http: &serenity::Http, data: &ReminderData) -> Result<()> {
    let user = data.user.to_user(&http).await?;

    data.channel
        .send_message(
            http,
            serenity::CreateMessage::default()
                .content(data.user.mention().to_string())
                .embed(
                    serenity::CreateEmbed::default()
                        .title("Reminder")
                        .description(
                            data.content
                                .clone()
                                .unwrap_or_else(|| "*No content*".into()),
                        )
                        .author(serenity::CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
                        .color(0x3bc9db),
                ),
        )
        .await?;

    Ok(())
}

/// Create a reminder for yourself
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only)]
pub async fn remind(
    ctx: Context<'_>,
    #[description = "Duration"] duration: String,
    #[description = "Content"] content: Option<String>,
) -> Result<()> {
    ctx.defer().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        if let Some(member) = ctx.author_member().await {
            if let Ok(serenity::Channel::Guild(channel)) =
                ctx.channel_id().to_channel(&ctx, ctx.guild_id()).await
            {
                if ctx.guild().is_some_and(|guild| {
                    guild.user_permissions_in(&channel, &member).send_messages()
                }) {
                    let timestamp = chrono::Utc::now() + duration;

                    let data = ReminderData {
                        guild: ctx
                            .guild_id()
                            .ok_or_else(|| eyre!("could not obtain guild ID"))?,
                        channel: ctx.channel_id(),
                        user: member.user.id,
                        content: content.clone(),
                        timestamp,
                    };

                    task::spawn({
                        let http = ctx.serenity_context().http.clone();
                        let data = data.clone();

                        async move {
                            time::sleep(duration).await;
                            if let Err(err) = dispatch(&http, &data).await {
                                tracing::error!("{err:?}");
                            }
                        }
                    });

                    ctx.send(
                        poise::CreateReply::default().embed(
                            serenity::CreateEmbed::default()
                                .title("Reminder set!")
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
                                    serenity::CreateEmbedAuthor::new(member.user.tag())
                                        .icon_url(member.face()),
                                )
                                .color(0x3bc9db),
                        ),
                    )
                    .await?;

                    if let Some(storage) = &ctx.data().storage {
                        storage.add_reminders(&data).await?;
                        storage.clean_reminders().await?;
                    }

                    return Ok(());
                }
            }
        }
    }

    ctx.say("Failed to set reminder!").await?;

    Ok(())
}

#[tracing::instrument(skip(ctx))]
pub async fn restore(ctx: &serenity::Context) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        let reminders = storage.scan_reminders().await?;

        for data in &reminders {
            task::spawn({
                let http = ctx.http.clone();
                let data = data.clone();
                let duration = (data.timestamp - chrono::Utc::now())
                    .to_std()
                    .unwrap_or_default();

                async move {
                    time::sleep(duration).await;
                    if let Err(err) = dispatch(&http, &data).await {
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
