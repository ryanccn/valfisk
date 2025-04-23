// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use serde::Deserialize;
use std::sync::LazyLock;

use poise::serenity_prelude::{GenericChannelId, GuildId, RoleId};

use crate::handlers::starboard::StarboardEmojis;

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub discord_token: String,
    pub redis_url: Option<String>,

    pub guild_id: Option<GuildId>,

    pub private_category: Option<GenericChannelId>,
    pub private_starboard_channel: Option<GenericChannelId>,
    pub starboard_channel: Option<GenericChannelId>,

    #[serde(default)]
    pub starboard_emojis: StarboardEmojis,
    #[serde(default = "defaults::starboard_threshold")]
    pub starboard_threshold: u64,

    pub moderation_logs_channel: Option<GenericChannelId>,
    pub message_logs_channel: Option<GenericChannelId>,
    pub member_logs_channel: Option<GenericChannelId>,
    pub dm_logs_channel: Option<GenericChannelId>,
    pub error_logs_channel: Option<GenericChannelId>,

    pub moderator_role: Option<RoleId>,
    #[serde(default)]
    pub logs_excluded_channels: Vec<GenericChannelId>,

    #[serde(default)]
    pub random_color_roles: Vec<RoleId>,
    #[serde(default)]
    pub intelligence_allowed_roles: Vec<RoleId>,

    pub pagespeed_api_key: Option<String>,
    pub safe_browsing_api_key: Option<String>,
    pub intelligence_secret: Option<String>,

    pub kofi_verification_token: Option<String>,
    pub kofi_notify_channel: Option<GenericChannelId>,

    #[serde(default = "defaults::host")]
    pub host: String,
    #[serde(default = "defaults::port")]
    pub port: u16,
}

pub static CONFIG: LazyLock<EnvConfig> =
    LazyLock::new(|| envy::from_env().expect("could not parse config from environment"));

mod defaults {
    pub fn host() -> String {
        #[cfg(debug_assertions)]
        return "127.0.0.1".into();

        #[cfg(not(debug_assertions))]
        return "0.0.0.0".into();
    }

    pub fn port() -> u16 {
        8080
    }

    pub fn starboard_threshold() -> u64 {
        3
    }
}
