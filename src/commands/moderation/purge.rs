// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Purge a number of messages from a channel
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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

    while count_remaining > 0 {
        let count_current: u8 = count_remaining.min(100).try_into()?;

        let messages = channel
            .messages(ctx, serenity::GetMessages::new().limit(count_current))
            .await?
            .iter()
            .filter(|m| {
                *m.timestamp >= chrono::Utc::now() - chrono::Duration::weeks(2)
                    && m.kind != serenity::MessageType::ThreadStarterMessage
            })
            .map(|m| m.id)
            .take(count_current.into())
            .collect::<Vec<_>>();

        if messages.is_empty() {
            break;
        }

        channel
            .delete_messages(
                ctx.http(),
                &messages,
                Some(&format!(
                    "Purge by @{} ({})",
                    ctx.author().name,
                    ctx.author().id
                )),
            )
            .await?;

        count_remaining -= u64::from(count_current);
        count_success += messages.len();
    }

    ctx.say(format!("**Success!** Deleted {count_success} messages."))
        .await?;

    Ok(())
}
