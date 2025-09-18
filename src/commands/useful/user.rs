// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::borrow::Cow;

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

fn short_link<'a>(url: impl Into<Cow<'a, str>>) -> String {
    format!("[View]({})", url.into())
}

/// Show information about a user
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn user(ctx: Context<'_>, user: serenity::UserId) -> Result<()> {
    ctx.defer().await?;

    let user = user.to_user(&ctx).await?;

    let mut embed = serenity::CreateEmbed::default()
        .title(&user.name)
        .field("ID", user.id.to_string(), true)
        .field(
            "Global name",
            user.global_name.as_ref().map_or("*None*", |s| s.as_str()),
            true,
        )
        .field("Avatar", short_link(user.face()), true)
        .field(
            "Banner",
            user.banner_url()
                .map_or_else(|| "*None*".to_owned(), |u| short_link(&u)),
            true,
        )
        .field(
            "Accent color",
            user.accent_colour
                .map_or("*None*".to_owned(), |c| format!("#{}", c.hex())),
            true,
        )
        .field(
            "Flags",
            if user.flags.is_empty() {
                "*None*".to_owned()
            } else {
                user.flags
                    .iter_names()
                    .map(|(n, _)| format!("`{n}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            false,
        )
        .field(
            "Created at",
            format!("<t:{0}:F> (<t:{0}:R>)", user.id.created_at().timestamp()),
            false,
        )
        .thumbnail(user.face());

    if let Some(color) = &user.accent_colour {
        embed = embed.color(*color);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
