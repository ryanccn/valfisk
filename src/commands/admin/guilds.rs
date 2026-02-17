// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use futures_util::future::try_join_all;
use poise::{CreateReply, serenity_prelude as serenity};

use eyre::Result;

use crate::{Context, utils::serenity::format_mentionable};

#[derive(Clone, Debug)]
struct RichGuildInfo {
    pub id: serenity::GuildId,
    pub name: String,
    pub owner: serenity::UserId,
    pub members: Option<u64>,
}

/// List guilds that have the app installed
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    default_member_permissions = "ADMINISTRATOR",
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn guilds(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guilds = try_join_all(
        ctx.http()
            .get_guilds(None, Some(200.try_into()?))
            .await?
            .iter()
            .map(|guild| async move {
                let partial = guild.id.to_partial_guild_with_counts(ctx.http()).await?;

                eyre::Ok(RichGuildInfo {
                    id: guild.id,
                    name: guild.name.to_string(),
                    owner: partial.owner_id,
                    members: partial.approximate_member_count.map(|c| c.get()),
                })
            }),
    )
    .await?;

    ctx.send(
        CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(&[serenity::CreateComponent::Container(
                serenity::CreateContainer::new(&[
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!("## Guilds ({})", guilds.len())),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(
                            guilds
                                .iter()
                                .map(|guild| {
                                    format!(
                                        "### {} (`{}`)\n**Owner**: {}\n**Members**: {}",
                                        guild.name,
                                        guild.id,
                                        format_mentionable(Some(guild.owner)),
                                        guild.members.map_or_else(
                                            || "Unknown".to_string(),
                                            |c| c.to_string()
                                        )
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ),
                    ),
                ])
                .accent_color(0x4ade80),
            )]),
    )
    .await?;

    Ok(())
}
