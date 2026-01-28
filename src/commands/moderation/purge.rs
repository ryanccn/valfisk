use eyre::Result;
use futures_util::StreamExt;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Delete multiple messages in bulk
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_MESSAGES"
)]
pub async fn purge(ctx: Context<'_>, count: usize) -> Result<()> {
    let mut ids: Vec<serenity::MessageId> = vec![];
    let mut iter = ctx
        .channel_id()
        .messages_iter(ctx.http())
        .take(count)
        .boxed();

    while let Some(message) = iter.next().await {
        let message = message?;

        if *message.timestamp < chrono::Utc::now() - chrono::Duration::weeks(2) {
            break;
        }

        if message.kind == serenity::MessageType::ThreadStarterMessage {
            // thread starter messages cannot be deleted
            continue;
        }

        ids.push(message.id);
    }

    ctx.channel_id()
        .delete_messages(ctx.http(), &ids, None)
        .await?;

    ctx.say(format!("Deleted {} messages!", ids.len())).await?;

    Ok(())
}
