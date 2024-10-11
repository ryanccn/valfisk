// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use once_cell::sync::Lazy;
use poise::serenity_prelude::ChannelId;

pub mod ban;
pub mod kick;
pub mod timeout;

static LOGS_CHANNEL: Lazy<Option<ChannelId>> = Lazy::new(|| {
    std::env::var("MODERATION_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<ChannelId>().ok())
});
