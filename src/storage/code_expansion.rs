// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeExpansionData {
    pub message: serenity::MessageId,
    pub content_hash: String,
}
