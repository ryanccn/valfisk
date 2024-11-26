// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use serde::Deserialize;
use std::sync::LazyLock;

use poise::serenity_prelude::{ChannelId, GuildId, RoleId};

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub discord_token: String,
    pub redis_url: Option<String>,

    pub guild_id: Option<GuildId>,

    pub fren_category: Option<ChannelId>,
    pub fren_starboard_channel: Option<ChannelId>,
    pub starboard_channel: Option<ChannelId>,

    pub moderation_logs_channel: Option<ChannelId>,
    pub dm_logs_channel: Option<ChannelId>,
    pub message_logs_channel: Option<ChannelId>,
    pub member_logs_channel: Option<ChannelId>,
    pub error_logs_channel: Option<ChannelId>,

    pub moderator_role: Option<RoleId>,

    #[serde(default)]
    pub random_color_roles: Vec<RoleId>,
    #[serde(default)]
    pub intelligence_allowed_roles: Vec<RoleId>,

    pub pagespeed_api_key: Option<String>,
    pub safe_browsing_api_key: Option<String>,
    pub intelligence_secret: Option<String>,

    pub kofi_verification_token: Option<String>,
    pub kofi_notify_channel: Option<ChannelId>,
}

pub static CONFIG: LazyLock<EnvConfig> =
    LazyLock::new(|| envy::from_env().expect("could not parse config from environment"));
