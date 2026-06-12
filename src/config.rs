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

    pub intelligence_allowed_roles: Option<HashSet<RoleId>>,
    pub anthropic_api_key: Option<String>,

    pub umami_endpoint: Option<String>,
    pub umami_website_id: Option<String>,
    pub umami_hostname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GuildConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_category: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_starboard_channel: Option<GenericChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_starboard_threshold: Option<u64>,

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_extra_message_ban: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_extra_message_kick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_extra_message_timeout: Option<String>,
}

pub static CONFIG: LazyLock<EnvConfig> =
    LazyLock::new(|| envy::from_env().expect("could not parse config from environment"));
