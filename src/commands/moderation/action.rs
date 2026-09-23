// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Display;

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::{Context, config::GuildConfig, utils};

pub fn container(
    title: &str,
    user: serenity::UserId,
    accent_color: u32,
) -> serenity::CreateContainer<'static> {
    serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "### {title}\n{}",
            utils::serenity::format_mentionable(Some(user))
        )),
    )])
    .accent_color(accent_color)
}

pub fn field(
    container: serenity::CreateContainer<'static>,
    name: &str,
    value: impl Display,
) -> serenity::CreateContainer<'static> {
    container.add_component(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("**{name}**\n{value}")),
    ))
}

/// Posts the action to the configured moderation logs channel.
pub async fn log(
    http: &serenity::Http,
    guild_config: &GuildConfig,
    container: serenity::CreateContainer<'static>,
    moderator: serenity::UserId,
    source: Option<&str>,
) -> Result<()> {
    if let Some(logs_channel) = guild_config.moderation_logs_channel {
        let log_container =
            container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "-# {}{} \u{00B7} {}",
                    moderator.mention(),
                    source.map_or_else(String::new, |s| format!(" \u{00B7} {s}")),
                    serenity::FormattedTimestamp::now()
                )),
            ));

        logs_channel
            .send_message(
                http,
                serenity::CreateMessage::default()
                    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                    .allowed_mentions(serenity::CreateAllowedMentions::new())
                    .components(vec![serenity::CreateComponent::Container(log_container)]),
            )
            .await?;
    }

    Ok(())
}

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

        Ok(ModerationAction {
            ctx,
            guild,
            guild_config,
            container: container(title, user, accent_color),
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
        self.container = field(self.container, name, value);
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

    pub async fn log(&self) -> Result<()> {
        if let Some(guild_config) = &self.guild_config {
            log(
                self.ctx.http(),
                guild_config,
                self.container.clone(),
                self.ctx.author().id,
                None,
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
