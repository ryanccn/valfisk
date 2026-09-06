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

        let channels = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::ChannelSelect { values } => Some(values),
            _ => None,
        };

        let roles = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::RoleSelect { values } => Some(values),
            _ => None,
        };

        macro_rules! set_first_channel {
            ($field:expr) => {
                if let Some(values) = channels {
                    $field = values.first().map(|ch| ch.widen());
                }
            };
        }

        match config_key {
            "private_category" => set_first_channel!(config.private_category),
            "private_starboard_channel" => set_first_channel!(config.private_starboard_channel),
            "starboard_channel" => set_first_channel!(config.starboard_channel),
            "moderation_logs_channel" => set_first_channel!(config.moderation_logs_channel),
            "message_logs_channel" => set_first_channel!(config.message_logs_channel),
            "member_logs_channel" => set_first_channel!(config.member_logs_channel),
            "honeypot_channel" => set_first_channel!(config.honeypot_channel),

            "moderator_role" => {
                if let Some(values) = roles {
                    config.moderator_role = values.first().copied();
                }
            }

            "logs_excluded_channels" => {
                if let Some(values) = channels {
                    config.logs_excluded_channels =
                        values.iter().map(|ch| ch.widen()).collect::<HashSet<_>>();
                }
            }
            "random_color_roles" => {
                if let Some(values) = roles {
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
