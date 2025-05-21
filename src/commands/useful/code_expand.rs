// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::CreateReply;

use crate::{Context, handlers::code_expansion};

/// Expand a link to lines of source code on GitHub, Codeberg, GitLab, and the Rust and Go playgrounds.
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    rename = "code-expand",
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn code_expand(
    ctx: Context<'_>,
    #[description = "A link, or multiple links"] content: String,
) -> Result<()> {
    let embeds = code_expansion::resolve(&content).await?;

    if embeds.is_empty() {
        ctx.say("No supported code links detected!").await?;
    } else {
        let mut reply = CreateReply::new();
        for embed in embeds {
            reply = reply.embed(embed);
        }
        ctx.send(reply).await?;
    }

    Ok(())
}
