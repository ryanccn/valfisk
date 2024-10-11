// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Translates a message
#[poise::command(context_menu_command = "Suppress Embeds", guild_only, ephemeral)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[allow(clippy::redundant_closure_for_method_calls)]
pub async fn suppress_embeds(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if ctx.author() == &message.author
        || ctx.author_member().await.is_some_and(|m| {
            m.permissions(ctx.cache())
                .is_ok_and(|p| p.manage_messages())
        })
    {
        let suppressed = message
            .flags
            .is_some_and(|flags| flags.contains(serenity::MessageFlags::SUPPRESS_EMBEDS));

        ctx.http()
            .edit_message(
                message.channel_id,
                message.id,
                &serenity::EditMessage::new().suppress_embeds(!suppressed),
                Vec::new(),
            )
            .await?;

        ctx.say(format!(
            "Embeds have been {} on this message.",
            if suppressed {
                "unsuppressed"
            } else {
                "suppressed"
            }
        ))
        .await?;
    } else {
        ctx.say("You do not have permissions to suppress embeds on this message!")
            .await?;
    }

    Ok(())
}
