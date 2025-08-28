// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{CreateReply, serenity_prelude as serenity};

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
    let components = code_expansion::resolve(&content).await?;

    if components.is_empty() {
        ctx.say("No supported code links detected!").await?;
    } else {
        ctx.send(
            CreateReply::new()
                .components(components)
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2),
        )
        .await?;
    }

    Ok(())
}
