// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use eyre::Result;

use crate::{config::CONFIG, utils};

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if message.channel(&ctx).await?.private().is_some()
        && let Some(logs_channel) = CONFIG.dm_logs_channel {
            let mut embed = serenity::CreateEmbed::default()
                .description(message.content.clone())
                .author(
                    serenity::CreateEmbedAuthor::new(message.author.tag())
                        .icon_url(message.author.face()),
                )
                .color(0x9775fa)
                .timestamp(message.timestamp);

            if !message.attachments.is_empty() {
                embed = embed.field(
                    "Attachments",
                    utils::serenity::format_attachments(&message.attachments),
                    false,
                );
            }

            logs_channel
                .send_message(&ctx.http, serenity::CreateMessage::default().embed(embed))
                .await?;
        }

    Ok(())
}
