// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use std::{fmt::Display, sync::Arc, time::Duration};
use tokio::time::timeout;

use poise::serenity_prelude::{self as serenity, Mentionable};

use crate::utils;

pub struct PartialContext {
    cache: Arc<serenity::Cache>,
    http: Arc<serenity::Http>,
}

impl serenity::CacheHttp for PartialContext {
    fn http(&self) -> &serenity::Http {
        &self.http
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}

impl From<&serenity::Context> for PartialContext {
    fn from(value: &serenity::Context) -> Self {
        Self {
            cache: Arc::clone(&value.cache),
            http: Arc::clone(&value.http),
        }
    }
}

pub async fn suppress_embeds(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    use futures_util::StreamExt as _;
    use std::time::Duration;
    use tokio::time::timeout;

    let mut message_updates = serenity::collect(ctx, {
        let id = message.id;
        move |ev| match ev {
            serenity::Event::MessageUpdate(x) if x.message.id == id => Some(()),
            _ => None,
        }
    });

    let _ = timeout(Duration::from_millis(2500), message_updates.next()).await;

    ctx.http
        .edit_message(
            message.channel_id,
            message.id,
            &serenity::EditMessage::new().suppress_embeds(true),
            Vec::new(),
        )
        .await?;

    Ok(())
}

pub fn format_mentionable(id: Option<impl Mentionable + Display>) -> String {
    id.map_or_else(
        || "*Unknown*".to_owned(),
        |id| format!("{} `{id}`", id.mention()),
    )
}

pub fn format_attachments(attachments: &[serenity::Attachment]) -> String {
    attachments
        .iter()
        .map(|att| {
            format!(
                "[{}]({}) ({})",
                att.filename,
                att.url,
                utils::format_bytes(att.size.into())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn interaction_confirm<'a>(
    ctx: &crate::Context<'a>,
    embed: serenity::CreateEmbed<'_>,
) -> Result<(bool, poise::ReplyHandle<'a>)> {
    use futures_util::StreamExt as _;

    let confirm_button_id = utils::nanoid(16);
    let cancel_button_id = utils::nanoid(16);

    let handle = ctx
        .send(poise::CreateReply::default().embed(embed).components(
            vec![serenity::CreateActionRow::Buttons(
                    vec![
                        serenity::CreateButton::new(&confirm_button_id)
                            .label("Confirm")
                            .style(serenity::ButtonStyle::Primary),
                        serenity::CreateButton::new(&cancel_button_id)
                            .label("Cancel")
                            .style(serenity::ButtonStyle::Secondary),
                    ]
                    .into(),
                )],
        ))
        .await?;

    let interaction = timeout(
        Duration::from_secs(24 * 60 * 60),
        serenity::collect(ctx.serenity_context(), {
            let confirm_message_id = handle.message().await?.id;

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
        interaction.defer(ctx.http()).await?;

        if interaction.data.custom_id == confirm_button_id {
            return Ok((true, handle));
        }
    }

    Ok((false, handle))
}
