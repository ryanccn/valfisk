// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::LazyLock};

use poise::serenity_prelude::{GenericChannelId, GuildId, RoleId, UserId};

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub discord_token: String,
    pub redis_url: Option<String>,

    pub allowed_guilds: Option<HashSet<GuildId>>,

    pub admin_guild_id: Option<GuildId>,
    pub owners: Option<HashSet<UserId>>,
    pub error_logs_channel: Option<GenericChannelId>,
    pub dm_logs_channel: Option<GenericChannelId>,

    pub pagespeed_api_key: Option<String>,
    pub safe_browsing_api_key: Option<String>,
    pub translation_api_key: Option<String>,

    pub intelligence_allowed_roles: Option<HashSet<RoleId>>,
    pub openrouter_api_key: Option<String>,

    #[serde(default = "defaults::host")]
    pub host: String,
    #[serde(default = "defaults::port")]
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GuildConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_category: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_starboard_channel: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starboard_channel: Option<GenericChannelId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub starboard_emojis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starboard_threshold: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_logs_channel: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_logs_channel: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_logs_channel: Option<GenericChannelId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderator_role: Option<RoleId>,
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub logs_excluded_channels: HashSet<GenericChannelId>,

    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub random_color_roles: HashSet<RoleId>,
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
}
