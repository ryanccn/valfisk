// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{self as serenity, Mentionable as _},
};

use crate::{Context, schedule};

/// Rotate color roles to a random color
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    ephemeral,
    rename = "rotate-color-roles",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn rotate_color_roles(
    ctx: Context<'_>,
    #[description = "The role color to rotate"] role: Option<serenity::RoleId>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let roles = schedule::rotate_color_roles(&ctx.serenity_context().http, role).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Rotated color roles!")
                .description(
                    roles
                        .iter()
                        .map(|r| r.mention().to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
                .color(0x69db7c),
        ),
    )
    .await?;

    Ok(())
}
