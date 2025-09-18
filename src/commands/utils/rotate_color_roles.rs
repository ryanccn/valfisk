// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
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

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let roles = if let Some(role) = role {
        schedule::rotate_color_role(ctx.http(), guild_id, role).await?
    } else {
        schedule::rotate_color_roles_guild(ctx.http(), &ctx.data(), guild_id).await?
    };

    ctx.send(
        CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(&[serenity::CreateComponent::Container(
                serenity::CreateContainer::new(&[serenity::CreateComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "### Rotated color roles\n{}",
                        roles
                            .iter()
                            .map(|r| r.mention().to_string())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )),
                )])
                .accent_color(0x69db7c),
            )]),
    )
    .await?;

    Ok(())
}
