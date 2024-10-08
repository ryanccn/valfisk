use poise::serenity_prelude as serenity;

use eyre::Result;
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::{task, time};

use crate::{intelligence, utils::GUILD_ID};

static ALLOWED_ROLES: Lazy<Vec<serenity::RoleId>> = Lazy::new(|| {
    std::env::var("INTELLIGENCE_ALLOWED_ROLES")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|f| f.trim().parse::<serenity::RoleId>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.guild_id != *GUILD_ID {
        return Ok(());
    }

    if message
        .flags
        .is_some_and(|flags| flags.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
    {
        return Ok(());
    }

    if let Ok(member) = message.member(&ctx).await {
        if !member.permissions(&ctx.cache)?.administrator()
            && !member.roles.iter().any(|r| ALLOWED_ROLES.contains(r))
        {
            return Ok(());
        }

        if let Some(query) = message
            .content
            .strip_prefix(&format!("<@{}>", ctx.cache.current_user().id))
            .map(|s| s.trim())
        {
            if query.is_empty() {
                return Ok(());
            }

            let typing_task = task::spawn({
                let http = ctx.http.clone();
                let channel = message.channel_id;

                async move {
                    let _ = time::timeout(Duration::from_secs(60), async move {
                        let mut interval = time::interval(Duration::from_secs(10));

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
