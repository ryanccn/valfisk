use std::time::Duration;

use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;
use tokio::{task, time};

use crate::intelligence;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    if message
        .flags
        .is_some_and(|flags| flags.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
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
                let mut interval = time::interval(Duration::from_secs(10));

                loop {
                    interval.tick().await;
                    let _ = http.broadcast_typing(channel).await;
                }
            }
        });

        let username = message.author.tag();
        let display_name = message.author.global_name.clone().map(|s| s.into_string());

        let nick = message
            .member
            .as_ref()
            .and_then(|m| m.nick.as_ref().map(|s| s.to_owned().into_string()));

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

    Ok(())
}
