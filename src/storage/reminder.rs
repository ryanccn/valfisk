// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use crate::impl_redis_serde;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReminderData {
    pub guild: serenity::GuildId,
    pub channel: serenity::GenericChannelId,
    pub user: serenity::UserId,
    pub content: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl_redis_serde!(ReminderData);
