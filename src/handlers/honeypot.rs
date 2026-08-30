// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::{analytics, utils};

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(false);
    }

    if let Some(guild_id) = message.guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage
        && let Ok(config) = storage.get_config(guild_id).await
        && let Some(honeypot_channel) = config.honeypot_channel
        && honeypot_channel == message.channel_id
    {
        let purged = async {
            guild_id
                .ban(
                    &ctx.http,
                    message.author.id,
                    3600,
                    Some("Honeypot triggered"),
                )
                .await?;

            guild_id
                .unban(&ctx.http, message.author.id, Some("Honeypot triggered"))
                .await?;

            eyre::Ok(())
        }
        .await
        .is_ok();

        if let Some(logs_channel) = config.message_logs_channel {
            let mut components: Vec<serenity::CreateComponent<'_>> = Vec::new();

            // if let Some(role) = config.moderator_role {
            //     components.push(serenity::CreateComponent::TextDisplay(
            //         serenity::CreateTextDisplay::new(role.mention().to_string()),
            //     ));
            // }

            let mut container = serenity::CreateContainer::new(vec![
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    "### Honeypot",
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Author**\n{} (*{}*)",
                        utils::serenity::format_mentionable(Some(message.author.id)),
                        if purged { "purged" } else { "purge failed" }
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Channel**\n{}",
                        utils::serenity::format_mentionable(Some(message.channel_id))
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!("**Content**\n{}", utils::truncate(&message.content, 1024)),
                )),
            ])
            .accent_color(0xff6b6b);

            if !message.attachments.is_empty() {
                container =
                    container.add_component(serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Attachments**\n{}",
                            utils::serenity::format_attachments(&message.attachments),
                        )),
                    ));
            }

            let image_attachments = message
                .attachments
                .iter()
                .filter(|att| {
                    att.content_type
                        .as_ref()
                        .is_some_and(|ct| ct.starts_with("image/"))
                })
                .take(10)
                .collect::<Vec<_>>();

            if !image_attachments.is_empty() {
                container =
                    container.add_component(serenity::CreateContainerComponent::MediaGallery(
                        serenity::CreateMediaGallery::new(
                            image_attachments
                                .iter()
                                .map(|attachment| {
                                    serenity::CreateMediaGalleryItem::new(
                                        serenity::CreateUnfurledMediaItem::new(&attachment.url),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        ),
                    ));
            }

            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "-# {}",
                    serenity::FormattedTimestamp::now()
                )),
            ));

            components.push(serenity::CreateComponent::Container(container));

            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(
                            serenity::CreateAllowedMentions::new()
                                .roles(config.moderator_role.iter().copied().collect::<Vec<_>>()),
                        )
                        .components(&components),
                )
                .await?;
        }

        if !purged {
            message
                .delete(&ctx.http, Some("Honeypot triggered"))
                .await?;
        }

        analytics::send_honeypot(message.guild_id).await;

        return Ok(true);
    }

    Ok(false)
}
