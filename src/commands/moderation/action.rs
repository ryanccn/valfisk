// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Display;

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, config::GuildConfig, utils};

pub struct ModerationAction<'a> {
    ctx: Context<'a>,
    guild: serenity::PartialGuild,
    guild_config: Option<GuildConfig>,
    container: serenity::CreateContainer<'static>,
}

impl<'a> ModerationAction<'a> {
    pub async fn new(
        ctx: Context<'a>,
        title: &str,
        user: serenity::UserId,
        accent_color: u32,
    ) -> Result<ModerationAction<'a>> {
        let guild = ctx
            .partial_guild()
            .await
            .ok_or_else(|| eyre!("failed to obtain partial guild"))?;

        let guild_config = if let Some(storage) = &ctx.data().storage {
            Some(storage.get_config(guild.id).await?)
        } else {
            None
        };

        let container =
            serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "### {title}\n{}",
                    utils::serenity::format_mentionable(Some(user))
                )),
            )])
            .accent_color(accent_color);

        Ok(ModerationAction {
            ctx,
            guild,
            guild_config,
            container,
        })
    }

    pub const fn guild(&self) -> &serenity::PartialGuild {
        &self.guild
    }

    pub const fn guild_config(&self) -> Option<&GuildConfig> {
        self.guild_config.as_ref()
    }

    #[must_use]
    pub fn field(mut self, name: &str, value: impl Display) -> Self {
        self.container =
            self.container
                .add_component(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!("**{name}**\n{value}")),
                ));

        self
    }

    /// Sends the action summary to the target as a direct message and records
    /// the outcome. This runs before the action itself, since a banned or
    /// kicked user can no longer be messaged.
    #[must_use]
    pub async fn notify(self, user: &serenity::User, dm: bool) -> Self {
        if !dm {
            return self.field("User notified", "No");
        }

        let dm_container =
            self.container
                .clone()
                .add_component(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "-# {} \u{00B7} {}",
                        self.guild.name,
                        serenity::FormattedTimestamp::now()
                    )),
                ));

        let notified = if let Ok(dm) = user.create_dm_channel(self.ctx).await {
            dm.id
                .widen()
                .send_message(
                    self.ctx.http(),
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(vec![serenity::CreateComponent::Container(dm_container)]),
                )
                .await
                .is_ok()
        } else {
            false
        };

        self.field("User notified", if notified { "Yes" } else { "Failed" })
    }

    /// Posts the action to the configured moderation logs channel.
    pub async fn log(&self) -> Result<()> {
        if let Some(guild_config) = &self.guild_config
            && let Some(logs_channel) = guild_config.moderation_logs_channel
        {
            let log_container = self.container.clone().add_component(
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "-# {} \u{00B7} {}",
                        self.ctx.author().mention(),
                        serenity::FormattedTimestamp::now()
                    ),
                )),
            );

            logs_channel
                .send_message(
                    self.ctx.http(),
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(vec![serenity::CreateComponent::Container(log_container)]),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn reply(self) -> Result<()> {
        self.ctx
            .send(
                poise::CreateReply::default()
                    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                    .components(vec![serenity::CreateComponent::Container(self.container)]),
            )
            .await?;

        Ok(())
    }
}
