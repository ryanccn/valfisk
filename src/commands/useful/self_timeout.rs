// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude as serenity;

use crate::Context;

/// Time yourself out for a specific duration
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(rename = "self-timeout", slash_command, guild_only)]
pub async fn self_timeout(
    ctx: Context<'_>,
    #[description = "The duration to time yourself out for"] duration: String,
    #[description = "The reason for the timeout"] reason: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        if let Some(mut member) = ctx.author_member().await {
            let start = chrono::Utc::now();
            let end = start + duration;

            member
                .to_mut()
                .edit(
                    ctx.http(),
                    serenity::EditMember::default()
                        .disable_communication_until(end.into())
                        .audit_log_reason(&format!(
                            "Requested self timeout{}",
                            reason
                                .as_ref()
                                .map(|r| format!(": {r}"))
                                .unwrap_or_default()
                        )),
                )
                .await?;

            let resp_embed = serenity::CreateEmbed::default()
                .title("Self-timeout in effect")
                .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                .field(
                    "End",
                    format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                    false,
                )
                .color(0x4ade80);

            ctx.send(poise::CreateReply::default().embed(resp_embed.clone()))
                .await?;

            if let Some(storage) = &ctx.data().storage {
                if let Ok(guild_channel) =
                    ctx.channel_id().to_guild_channel(ctx, ctx.guild_id()).await
                {
                    if storage
                        .get_self_timeout_transparency(ctx.author().id.get())
                        .await?
                        .unwrap_or(false)
                        && ctx.guild().is_some_and(|guild| {
                            guild
                                .user_permissions_in(&guild_channel, &member)
                                .send_messages()
                        })
                    {
                        let mut resp_embed = resp_embed.author(
                            serenity::CreateEmbedAuthor::new(ctx.author().tag())
                                .icon_url(ctx.author().face()),
                        );

                        if let Some(reason) = reason {
                            resp_embed = resp_embed.field("Reason", reason, false);
                        }

                        guild_channel
                            .send_message(
                                ctx.http(),
                                serenity::CreateMessage::default().embed(resp_embed),
                            )
                            .await?;
                    };
                }
            }
        }
    } else {
        ctx.say("Invalid duration!").await?;
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
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn transparency(
    ctx: Context<'_>,
    #[description = "Whether transparency is on or off"] status: bool,
) -> Result<()> {
    ctx.data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available for the transparency feature"))?
        .set_self_timeout_transparency(ctx.author().id.get(), &status)
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
