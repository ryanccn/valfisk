// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::ActivityData;

use serde::{Deserialize, Serialize};

#[derive(poise::ChoiceParameter, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PresenceKind {
    Custom,
    Playing,
    Watching,
    Listening,
    Competing,
    Clear,
}

impl PresenceKind {
    #[must_use]
    pub fn with_content(self, content: &str) -> PresenceData {
        PresenceData {
            r#type: self,
            content: content.to_owned(),
        }
    }
}

impl Default for PresenceKind {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for PresenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Custom => "Custom",
            Self::Playing => "Playing",
            Self::Watching => "Watching",
            Self::Listening => "Listening",
            Self::Competing => "Competing",
            Self::Clear => "Clear",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresenceData {
    pub r#type: PresenceKind,
    pub content: String,
}

impl PresenceData {
    #[must_use]
    pub fn to_activity(&self) -> Option<ActivityData> {
        match self.r#type {
            PresenceKind::Custom => ActivityData::custom(&self.content).into(),
            PresenceKind::Playing => ActivityData::playing(&self.content).into(),
            PresenceKind::Watching => ActivityData::watching(&self.content).into(),
            PresenceKind::Listening => ActivityData::listening(&self.content).into(),
            PresenceKind::Competing => ActivityData::competing(&self.content).into(),
            PresenceKind::Clear => None,
        }
    }
}
