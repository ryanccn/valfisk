// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{self as serenity, Mentionable as _};

use eyre::{eyre, Result};
use std::time::Duration;
use tokio::time;

use crate::{config::CONFIG, intelligence, utils::spawn_abort_on_drop};

async fn is_administrator(ctx: &serenity::Context, message: &serenity::Message) -> Result<bool> {
    let member = message.member(&ctx).await?;

    let guild = message
        .guild(&ctx.cache)
        .ok_or_else(|| eyre!("could not obtain guild"))?;

    let default_channel = guild
        .default_channel(message.author.id)
        .ok_or_else(|| eyre!("could not obtain default guild channel"))?;

    Ok(guild
        .user_permissions_in(default_channel, &member)
        .administrator())
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if CONFIG.intelligence_secret.is_none() {
        return Ok(());
    }

    if message.guild_id != CONFIG.guild_id {
        return Ok(());
    }

    if message
        .flags
        .is_some_and(|flags| flags.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
    {
        return Ok(());
    }

    if let Ok(member) = message.member(&ctx).await {
        if is_administrator(ctx, message).await.ok() != Some(true)
            && !member
                .roles
                .iter()
                .any(|r| CONFIG.intelligence_allowed_roles.contains(r))
        {
            return Ok(());
        }

        let self_prefix = ctx.cache.current_user().mention().to_string();

        if let Some(query) = message.content.strip_prefix(&self_prefix).map(|s| s.trim()) {
            if query.is_empty() {
                return Ok(());
            }

            let typing_task = spawn_abort_on_drop({
                let http = ctx.http.clone();
                let channel = message.channel_id;

                async move {
                    let _ = time::timeout(Duration::from_secs(60), async move {
                        let mut interval = time::interval(Duration::from_secs(5));

                        loop {
                            interval.tick().await;
                            let _ = http.broadcast_typing(channel).await;
                        }
                    })
                    .await;
                }
            });

            let username = message.author.tag();
            let display_name = message.author.global_name.clone().map(|s| s.into_string());

            let nick = member.nick.as_ref().map(|s| s.to_owned().into_string());

            let resp = intelligence::query(intelligence::Request {
                query: query.to_owned(),
                metadata: intelligence::RequestMetadata {
                    username,
                    display_name,
                    nick,
                },
            })
            .await?;

            message.reply_ping(&ctx.http, resp).await?;
            typing_task.abort();
        }
    }

    Ok(())
}
