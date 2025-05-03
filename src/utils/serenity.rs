// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Display;

use eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable};

use crate::utils;

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
