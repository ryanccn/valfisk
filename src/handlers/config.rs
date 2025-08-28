// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::HashSet;

use eyre::{Result, bail, eyre};
use poise::serenity_prelude as serenity;

pub async fn handle(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<()> {
    if let Some(config_key) = interaction.data.custom_id.strip_prefix("cfg:") {
        let guild_id = interaction
            .guild_id
            .ok_or_else(|| eyre!("could not obtain guild ID"))?;

        if !guild_id
            .to_partial_guild(&ctx)
            .await?
            .member_permissions(
                interaction
                    .member
                    .as_ref()
                    .ok_or_else(|| eyre!("could not obtain interaction member"))?,
            )
            .manage_guild()
        {
            return Ok(());
        }

        interaction.defer(&ctx.http).await?;

        let data = ctx.data::<crate::Data>();
        let storage = data
            .storage
            .as_ref()
            .ok_or_else(|| eyre!("storage is not available"))?;

        let mut config = storage.get_config(guild_id).await?;

        match config_key {
            "private_category" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.private_category = values.first().map(|ch| ch.widen());
                }
            }
            "private_starboard_channel" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.private_starboard_channel = values.first().map(|ch| ch.widen());
                }
            }
            "starboard_channel" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.starboard_channel = values.first().map(|ch| ch.widen());
                }
            }
            "moderation_logs_channel" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.moderation_logs_channel = values.first().map(|ch| ch.widen());
                }
            }
            "message_logs_channel" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.message_logs_channel = values.first().map(|ch| ch.widen());
                }
            }
            "member_logs_channel" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.member_logs_channel = values.first().map(|ch| ch.widen());
                }
            }
            "moderator_role" => {
                if let serenity::ComponentInteractionDataKind::RoleSelect { values } =
                    &interaction.data.kind
                {
                    config.moderator_role = values.first().copied();
                }
            }
            "logs_excluded_channels" => {
                if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                    &interaction.data.kind
                {
                    config.logs_excluded_channels =
                        values.iter().map(|ch| ch.widen()).collect::<HashSet<_>>();
                }
            }
            "random_color_roles" => {
                if let serenity::ComponentInteractionDataKind::RoleSelect { values } =
                    &interaction.data.kind
                {
                    config.random_color_roles = values.iter().copied().collect::<HashSet<_>>();
                }
            }
            &_ => {
                bail!("invalid config key in interaction: {config_key}")
            }
        }

        storage.set_config(guild_id, &config).await?;
    }

    Ok(())
}
