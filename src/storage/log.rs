// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{Attachment, Message, UserId};

use crate::impl_redis_serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageLog {
    pub content: String,
    pub author: UserId,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

impl MessageLog {
    pub fn new(content: &str, author: UserId, attachments: Vec<Attachment>) -> Self {
        Self {
            content: content.to_owned(),
            author,
            attachments,
        }
    }
}

impl From<&Message> for MessageLog {
    fn from(value: &Message) -> Self {
        Self {
            content: value.content.to_string(),
            author: value.author.id,
            attachments: value.attachments.to_vec(),
        }
    }
}

impl_redis_serde!(MessageLog);
