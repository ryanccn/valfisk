// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{Attachment, Message, UserId};

use crate::impl_redis_serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageLog {
    pub content: Option<String>,
    pub author: Option<UserId>,
    pub attachments: Vec<Attachment>,
}

impl MessageLog {
    pub const fn new(
        content: Option<String>,
        author: Option<UserId>,
        attachments: Vec<Attachment>,
    ) -> Self {
        Self {
            content,
            author,
            attachments,
        }
    }
}

impl From<&Message> for MessageLog {
    fn from(value: &Message) -> Self {
        Self {
            content: Some(value.content.clone().into_string()),
            author: Some(value.author.id),
            attachments: value.attachments.to_vec(),
        }
    }
}

impl_redis_serde!(MessageLog);
