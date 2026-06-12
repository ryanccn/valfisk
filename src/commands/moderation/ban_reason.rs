// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::serenity_prelude as serenity;

use crate::{Context, utils};

/// Find the reason for a user's ban
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    rename = "ban-reason",
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn ban_reason(
    ctx: Context<'_>,
    #[description = "The user"] user: serenity::UserId,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild = ctx.guild_id().ok_or_else(|| eyre!("no available guild"))?;

    let Some(ban) = guild.get_ban(ctx.http(), user).await? else {
        ctx.say("User is not banned!").await?;
        return Ok(());
    };

    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(vec![serenity::CreateComponent::Container(
                serenity::CreateContainer::new(vec![
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "### Ban\n{}",
                            utils::serenity::format_mentionable(Some(user)),
                        )),
                    ),
                    serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Reason**\n{}",
                            ban.reason
                                .map_or_else(|| "*None*".to_owned(), |s| s.to_string())
                        )),
                    ),
                ])
                .accent_color(0xda77f2),
            )]),
    )
    .await?;

    Ok(())
}
