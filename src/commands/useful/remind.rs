// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{eyre, Result};
use poise::serenity_prelude as serenity;
use tokio::{task, time};

use crate::Context;

/// Create a reminder for yourself
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn remind(
    ctx: Context<'_>,
    #[description = "Duration"] duration: String,
    #[description = "Content"] content: Option<String>,
) -> Result<()> {
    ctx.defer().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        if let Some(member) = ctx.author_member().await {
            if let Some(channel) = ctx.guild_channel().await {
                let end = chrono::Utc::now() + duration;

                task::spawn({
                    let http = ctx.serenity_context().http.clone();
                    let author = ctx.author().id;
                    let content = content.clone();
                    let member = member.clone().into_owned();

                    async move {
                        time::sleep(duration).await;

                        if let Err(err) = channel
                            .send_message(
                                &http,
                                serenity::CreateMessage::default()
                                    .content(format!("<@!{author}>"))
                                    .embed(
                                        serenity::CreateEmbed::default()
                                            .title("Reminder")
                                            .description(
                                                content
                                                    .unwrap_or_else(|| "*No content*".to_owned()),
                                            )
                                            .author(
                                                serenity::CreateEmbedAuthor::new(member.user.tag())
                                                    .icon_url(member.face()),
                                            )
                                            .color(0x3bc9db),
                                    ),
                            )
                            .await
                        {
                            tracing::error!("{err}");
                        };
                    }
                });

                ctx.send(
                    poise::CreateReply::default().embed(
                        serenity::CreateEmbed::default()
                            .title("Reminder set!")
                            .field(
                                "Time",
                                format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
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
            } else {
                ctx.say("Error: Guild channel unavailable!").await?;
            }
        } else {
            ctx.say("Error: Member unavailable!").await?;
        };
    } else {
        ctx.say("Error: Invalid duration!").await?;
    };

    Ok(())
}

/// Configure self-timeout transparency
#[poise::command(
    rename = "self-timeout-transparency",
    slash_command,
    guild_only,
    ephemeral
)]
pub async fn transparency(
    ctx: Context<'_>,
    #[description = "Whether transparency is on or off"] status: bool,
) -> Result<()> {
    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available for the transparency feature"))?;

    storage
        .set_self_timeout_transparency(&ctx.author().id.to_string(), &status)
        .await?;

    let desc = if status {
        "Your self-timeouts will now be publicly logged to the channel that you ran the self-timeout in."
    } else {
        "Your self-timeouts will no longer be publicly logged."
    };

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Self-timeout transparency updated!")
                .description(desc)
                .color(0x4ade80),
        ),
    )
    .await?;

    Ok(())
}
