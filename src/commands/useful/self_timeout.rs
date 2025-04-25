// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use std::time::Duration;
use tokio::time::timeout;

use futures_util::StreamExt as _;
use nanoid::nanoid;
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
        let mut member = ctx
            .author_member()
            .await
            .ok_or_else(|| eyre!("could not find member for author"))?;

        let start = chrono::Utc::now();
        let end = start + duration;

        let confirm_button_id = nanoid!(16);
        let cancel_button_id = nanoid!(16);

        let confirm_reply = ctx
            .send(
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .title("Requesting self-timeout")
                            .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                            .field(
                                "End",
                                format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                                false,
                            )
                            .color(0xffd43b),
                    )
                    .components(vec![serenity::CreateActionRow::Buttons(
                        vec![
                            serenity::CreateButton::new(&confirm_button_id)
                                .label("Confirm")
                                .style(serenity::ButtonStyle::Danger),
                            serenity::CreateButton::new(&cancel_button_id)
                                .label("Cancel")
                                .style(serenity::ButtonStyle::Secondary),
                        ]
                        .into(),
                    )]),
            )
            .await?;

        let interaction = timeout(
            Duration::from_secs(24 * 60 * 60),
            serenity::collect(ctx.serenity_context(), {
                let confirm_message_id = confirm_reply.message().await?.id;

                move |event| match event {
                    serenity::Event::InteractionCreate(event) => event
                        .interaction
                        .as_message_component()
                        .take_if(|i| i.message.id == confirm_message_id)
                        .cloned(),
                    _ => None,
                }
            })
            .next(),
        )
        .await?;

        if let Some(interaction) = interaction {
            if interaction.data.custom_id == confirm_button_id {
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

                let info_embed = serenity::CreateEmbed::default()
                    .title("Self-timeout in effect")
                    .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                    .field(
                        "End",
                        format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                        false,
                    )
                    .color(0x4ade80);

                confirm_reply
                    .edit(
                        ctx,
                        poise::CreateReply::new()
                            .embed(info_embed.clone())
                            .components(vec![]),
                    )
                    .await?;

                if let Some(storage) = &ctx.data().storage {
                    if let Ok(serenity::Channel::Guild(guild_channel)) =
                        ctx.channel_id().to_channel(&ctx, ctx.guild_id()).await
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
                            let mut info_embed_extended = info_embed.author(
                                serenity::CreateEmbedAuthor::new(ctx.author().tag())
                                    .icon_url(ctx.author().face()),
                            );

                            if let Some(reason) = reason {
                                info_embed_extended =
                                    info_embed_extended.field("Reason", reason, false);
                            }

                            guild_channel
                                .send_message(
                                    ctx.http(),
                                    serenity::CreateMessage::default().embed(info_embed_extended),
                                )
                                .await?;
                        }
                    }
                }
            } else if interaction.data.custom_id == cancel_button_id {
                confirm_reply
                    .edit(
                        ctx,
                        poise::CreateReply::new()
                            .embed(
                                serenity::CreateEmbed::default()
                                    .title("Self-timeout cancelled")
                                    .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                                    .field(
                                        "End",
                                        format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                                        false,
                                    )
                                    .color(0xff6b6b),
                            )
                            .components(vec![]),
                    )
                    .await?;
            } else {
                tracing::error!(?interaction, "received unknown interaction");
            }
        }
    } else {
        ctx.say("Invalid duration!").await?;
    }

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
