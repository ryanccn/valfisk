// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Purge a number of messages from a channel
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_MESSAGES"
)]
pub async fn purge(
    ctx: Context<'_>,

    #[description = "Number of messages to delete"]
    #[min = 1]
    count: u64,

    #[description = "Channel to delete messages from (defaults to current channel)"]
    channel: Option<serenity::GenericChannelId>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let channel = channel.unwrap_or_else(|| ctx.channel_id());

    let mut count_remaining = count;
    let mut count_success = 0usize;
    let mut before: Option<serenity::MessageId> = None;

    while count_remaining > 0 {
        let count_current: u8 = count_remaining.min(100).try_into()?;

        let mut request = serenity::GetMessages::new().limit(count_current);
        if let Some(before) = before {
            request = request.before(before);
        }

        let fetched = channel.messages(ctx, request).await?;

        if fetched.is_empty() {
            break;
        }

        before = fetched.last().map(|m| m.id);

        let messages = fetched
            .iter()
            .filter(|m| {
                *m.timestamp >= chrono::Utc::now() - chrono::Duration::weeks(2)
                    && m.kind != serenity::MessageType::ThreadStarterMessage
            })
            .map(|m| m.id)
            .collect::<Vec<_>>();

        if messages.is_empty() {
            continue;
        }

        let reason = format!("Purge by @{} ({})", ctx.author().name, ctx.author().id);

        if messages.len() == 1 {
            channel
                .delete_message(ctx.http(), messages[0], Some(&reason))
                .await?;
        } else {
            channel
                .delete_messages(ctx.http(), &messages, Some(&reason))
                .await?;
        }

        count_remaining -= u64::try_from(messages.len())?;
        count_success += messages.len();
    }

    ctx.say(format!("**Success!** Deleted {count_success} messages."))
        .await?;

    Ok(())
}
