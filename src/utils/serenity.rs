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

    let msg_id = message.id;

    let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
        Event::MessageUpdate(x) if x.id == msg_id => Some(()),
        _ => None,
    });

    let _ = timeout(Duration::from_millis(2000), message_updates.next()).await;

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
