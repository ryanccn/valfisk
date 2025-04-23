// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

#[tracing::instrument(skip(ctx))]
pub async fn suppress_embeds(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    use poise::futures_util::StreamExt as _;
    use serenity::{EditMessage, Event};
    use std::time::Duration;
    use tokio::time::timeout;

    let mut message_updates = serenity::collect(ctx, {
        let id = message.id;
        move |ev| match ev {
            Event::MessageUpdate(x) if x.message.id == id => Some(()),
            _ => None,
        }
    });

    let _ = timeout(Duration::from_millis(2500), message_updates.next()).await;

    ctx.http
        .edit_message(
            message.channel_id,
            message.id,
            &EditMessage::new().suppress_embeds(true),
            Vec::new(),
        )
        .await?;

    Ok(())
}

pub fn is_administrator(ctx: &serenity::Context, member: &serenity::Member) -> bool {
    member.roles(&ctx.cache).is_some_and(|roles| {
        roles
            .iter()
            .any(|role| role.has_permission(serenity::Permissions::ADMINISTRATOR))
    })
}
