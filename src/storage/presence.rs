// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::ActivityData;

use crate::impl_redis_serde;
use serde::{Deserialize, Serialize};

#[derive(poise::ChoiceParameter, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PresenceChoice {
    Custom,
    Playing,
    Watching,
    Listening,
    Competing,
}

impl PresenceChoice {
    #[must_use]
    pub fn to_data(self, content: &str) -> PresenceData {
        PresenceData {
            r#type: self,
            content: content.to_owned(),
        }
    }

    #[must_use]
    pub fn to_activity(self, content: &str) -> ActivityData {
        match self {
            Self::Custom => ActivityData::custom(content),
            Self::Playing => ActivityData::playing(content),
            Self::Watching => ActivityData::watching(content),
            Self::Listening => ActivityData::listening(content),
            Self::Competing => ActivityData::competing(content),
        }
    }
}

impl Default for PresenceChoice {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for PresenceChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Custom => "Custom",
            Self::Playing => "Playing",
            Self::Watching => "Watching",
            Self::Listening => "Listening",
            Self::Competing => "Competing",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresenceData {
    pub r#type: PresenceChoice,
    pub content: String,
}

impl PresenceData {
    #[must_use]
    pub fn to_activity(&self) -> ActivityData {
        self.r#type.to_activity(&self.content)
    }
}

impl_redis_serde!(PresenceData);
