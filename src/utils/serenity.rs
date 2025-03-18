// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude as serenity;

#[tracing::instrument(skip(ctx))]
pub async fn suppress_embeds(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    use poise::futures_util::StreamExt as _;
    use serenity::{EditMessage, Event};
    use std::time::Duration;
    use tokio::time::timeout;

    let mut message_updates = serenity::collector::collect(ctx, {
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

pub fn is_administrator(ctx: &serenity::Context, member: &serenity::Member) -> Result<bool> {
    let guild = member
        .guild_id
        .to_guild_cached(&ctx.cache)
        .ok_or_else(|| eyre!("could not obtain guild"))?;

    let default_channel = guild
        .default_channel(member.user.id)
        .ok_or_else(|| eyre!("could not obtain default channel"))?;

    Ok(guild
        .user_permissions_in(default_channel, member)
        .administrator())
}
