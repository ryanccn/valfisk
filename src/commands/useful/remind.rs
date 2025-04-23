// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable as _};
use tokio::{task, time};

use crate::Context;

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
                    let end = chrono::Utc::now() + duration;

                    task::spawn({
                        let http = ctx.serenity_context().http.clone();
                        let author = ctx.author().id;
                        let content = content.clone();
                        let embed_author = serenity::CreateEmbedAuthor::new(member.user.tag())
                            .icon_url(member.face());

                        async move {
                            time::sleep(duration).await;

                            if let Err(err) =
                                channel
                                    .send_message(
                                        &http,
                                        serenity::CreateMessage::default()
                                            .content(author.mention().to_string())
                                            .embed(
                                                serenity::CreateEmbed::default()
                                                    .title("Reminder")
                                                    .description(content.unwrap_or_else(|| {
                                                        "*No content*".to_owned()
                                                    }))
                                                    .author(embed_author)
                                                    .color(0x3bc9db),
                                            ),
                                    )
                                    .await
                            {
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

                    return Ok(());
                }
            }
        }
    }

    ctx.say("Failed to set reminder!").await?;

    Ok(())
}
