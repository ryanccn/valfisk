use poise::serenity_prelude::{Message, UserId};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(FromRedisValue, ToRedisArgs, Serialize, Deserialize, Clone, Debug)]
pub struct MessageLog {
    pub content: Option<String>,
    pub author: Option<UserId>,
}

impl MessageLog {
    pub const fn new(content: Option<String>, author: Option<UserId>) -> Self {
        Self { content, author }
    }
}

impl From<Message> for MessageLog {
    fn from(value: Message) -> Self {
        Self {
            content: Some(value.content.into_string()),
            author: Some(value.author.id),
        }
    }
}

impl From<&Message> for MessageLog {
    fn from(value: &Message) -> Self {
        Self {
            content: Some(value.content.clone().into_string()),
            author: Some(value.author.id),
        }
    }
}
