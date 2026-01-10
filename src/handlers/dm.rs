// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{self as serenity, Mentionable as _};

use eyre::Result;

use crate::{config::CONFIG, utils};

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if message.channel(&ctx).await?.private().is_some()
        && let Some(logs_channel) = CONFIG.dm_logs_channel
    {
        let mut container =
            serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "-# {} \u{00B7} {}\n{}",
                    message.author.mention(),
                    serenity::FormattedTimestamp::new(message.timestamp, None),
                    message.content
                )),
            )])
            .accent_color(0x9775fa);

        if !message.attachments.is_empty() {
            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "**Attachments**\n{}",
                    utils::serenity::format_attachments(&message.attachments),
                )),
            ));
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                    .allowed_mentions(serenity::CreateAllowedMentions::new())
                    .components(&[serenity::CreateComponent::Container(container)]),
            )
            .await?;
    }

    Ok(())
}
