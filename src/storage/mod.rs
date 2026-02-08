// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{GenericChannelId, GuildId, MessageId, UserId};
use redis::{
    AsyncCommands as _, RedisResult,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use std::time::Duration;

use log::MessageLog;
use reminder::ReminderData;

use crate::{config::GuildConfig, handlers::intelligence::IntelligenceMessages};

pub mod log;
pub mod presence;
mod redis_util;
pub mod reminder;

#[non_exhaustive]
pub struct Storage {
    conn: ConnectionManager,
}

impl Storage {
    pub async fn redis(redis: redis::Client) -> RedisResult<Self> {
        let conn = ConnectionManager::new_with_config(
            redis,
            ConnectionManagerConfig::new()
                .set_connection_timeout(Some(Duration::from_secs(10)))
                .set_response_timeout(Some(Duration::from_secs(60)))
                .set_number_of_retries(3),
        )
        .await?;

        Ok(Self { conn })
    }
}

impl Storage {
    pub async fn size(&self) -> RedisResult<u64> {
        let mut conn = self.conn.clone();
        let keys: u64 = redis::cmd("DBSIZE").query_async(&mut conn).await?;
        Ok(keys)
    }
}

mod keys {
    use std::{borrow::Cow, fmt};

    use poise::serenity_prelude::{GenericChannelId, GuildId, MessageId, UserId};

    pub struct StorageKey {
        base: &'static str,
        parts: Option<Vec<String>>,
    }

    impl StorageKey {
        pub const fn new(base: &'static str) -> Self {
            Self { base, parts: None }
        }

        pub fn part<'a>(self, s: impl Into<Cow<'a, str>>) -> Self {
            let mut parts = self.parts.clone().unwrap_or_default();
            parts.push(s.into().into_owned());

            Self {
                base: self.base,
                parts: Some(parts),
            }
        }

        pub fn guild(self, id: GuildId) -> Self {
            self.part(format!("g{id}"))
        }

        pub fn channel(self, id: GenericChannelId) -> Self {
            self.part(format!("c{id}"))
        }

        pub fn message(self, id: MessageId) -> Self {
            self.part(format!("m{id}"))
        }

        pub fn user(self, id: UserId) -> Self {
            self.part(format!("u{id}"))
        }
    }

    impl fmt::Display for StorageKey {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}{}",
                self.base,
                self.parts
                    .as_ref()
                    .map_or_else(String::new, |parts| format!(":{}", parts.join(":")))
            )
        }
    }

    impl redis::ToRedisArgs for StorageKey {
        fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + redis::RedisWrite,
        {
            out.write_arg(self.to_string().as_bytes());
        }
    }

    impl redis::ToSingleRedisArg for StorageKey {}

    pub const GUILD_CONFIG: StorageKey = StorageKey::new("guild-config-v1");
    pub const PRESENCE: StorageKey = StorageKey::new("presence-v1");
    pub const STARBOARD: StorageKey = StorageKey::new("starboard-v2");
    pub const MESSAGE_LOG: StorageKey = StorageKey::new("message-log-v2");
    pub const REMINDERS: StorageKey = StorageKey::new("reminders-v1");
    pub const AUTOREPLY: StorageKey = StorageKey::new("autoreply-v2");
    pub const INTELLIGENCE_CONSENT: StorageKey = StorageKey::new("intelligence-consent-v1");
    pub const INTELLIGENCE_CONTEXT: StorageKey = StorageKey::new("intelligence-context-v2");
    pub const WARN_COUNT: StorageKey = StorageKey::new("warn-count-v1");
}

impl Storage {
    pub async fn get_config(&self, guild_id: GuildId) -> RedisResult<GuildConfig> {
        let mut conn = self.conn.clone();
        let ret: Option<GuildConfig> = conn.get(keys::GUILD_CONFIG.guild(guild_id)).await?;
        Ok(ret.unwrap_or_default())
    }

    pub async fn set_config(&self, guild_id: GuildId, value: &GuildConfig) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.set(keys::GUILD_CONFIG.guild(guild_id), value).await?;
        Ok(())
    }

    pub async fn del_config(&self, guild_id: GuildId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::GUILD_CONFIG.guild(guild_id)).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_presence(&self) -> RedisResult<Option<presence::PresenceData>> {
        let mut conn = self.conn.clone();
        let ret: Option<presence::PresenceData> = conn.get(keys::PRESENCE).await?;
        Ok(ret)
    }

    pub async fn set_presence(&self, value: &presence::PresenceData) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.set(keys::PRESENCE, value).await?;
        Ok(())
    }

    pub async fn del_presence(&self) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::PRESENCE).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_starboard(&self, message_id: MessageId) -> RedisResult<Option<u64>> {
        let mut conn = self.conn.clone();
        let ret: Option<u64> = conn.get(keys::STARBOARD.message(message_id)).await?;
        Ok(ret)
    }

    pub async fn set_starboard(&self, message_id: MessageId, value: &u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn
            .set_options(
                keys::STARBOARD.message(message_id),
                value,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(2592000)),
            )
            .await?;
        Ok(())
    }

    pub async fn del_starboard(&self, message_id: MessageId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::STARBOARD.message(message_id)).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_message_log(&self, message_id: MessageId) -> RedisResult<Option<MessageLog>> {
        let mut conn = self.conn.clone();
        let ret: Option<MessageLog> = conn.get(keys::MESSAGE_LOG.message(message_id)).await?;
        Ok(ret)
    }

    pub async fn set_message_log(
        &self,
        message_id: MessageId,
        value: &MessageLog,
    ) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn
            .set_options(
                keys::MESSAGE_LOG.message(message_id),
                value,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(86400)),
            )
            .await?;
        Ok(())
    }

    pub async fn del_message_log(&self, message_id: MessageId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::MESSAGE_LOG.message(message_id)).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn scan_reminders(&self) -> RedisResult<Vec<ReminderData>> {
        use futures_util::StreamExt as _;

        let mut conn = self.conn.clone();

        let mut cmd = redis::cmd("ZSCAN");
        cmd.arg(keys::REMINDERS).cursor_arg(0).arg("NOSCORES");

        let values: Vec<ReminderData> = cmd
            .iter_async::<ReminderData>(&mut conn)
            .await?
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<RedisResult<Vec<_>>>()?;

        Ok(values)
    }

    pub async fn add_reminders(&self, value: &ReminderData) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn
            .zadd(keys::REMINDERS, value, value.timestamp.timestamp())
            .await?;

        Ok(())
    }

    pub async fn clean_reminders(&self) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn
            .zrembyscore(keys::REMINDERS, 0, chrono::Utc::now().timestamp() - 1)
            .await?;

        Ok(())
    }
}

impl Storage {
    pub async fn scan_autoreply(&self, guild_id: GuildId) -> RedisResult<Vec<(String, String)>> {
        use futures_util::StreamExt as _;

        let mut conn = self.conn.clone();
        let values: Vec<(String, String)> = conn
            .hscan(keys::AUTOREPLY.guild(guild_id))
            .await?
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<RedisResult<Vec<_>>>()?;

        Ok(values)
    }

    pub async fn add_autoreply(&self, guild_id: GuildId, f: &str, v: &str) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.hset(keys::AUTOREPLY.guild(guild_id), f, v).await?;
        Ok(())
    }

    pub async fn del_autoreply(&self, guild_id: GuildId, f: &str) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.hdel(keys::AUTOREPLY.guild(guild_id), f).await?;
        Ok(())
    }

    pub async fn delall_autoreply(&self, guild_id: GuildId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::AUTOREPLY.guild(guild_id)).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_intelligence_consent(&self, user_id: UserId) -> RedisResult<bool> {
        let mut conn = self.conn.clone();
        let value: bool = conn
            .sismember(keys::INTELLIGENCE_CONSENT, user_id.get())
            .await?;

        Ok(value)
    }

    pub async fn add_intelligence_consent(&self, user_id: UserId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.sadd(keys::INTELLIGENCE_CONSENT, user_id.get()).await?;

        Ok(())
    }
}

impl Storage {
    pub async fn get_intelligence_context(
        &self,
        user: UserId,
        channel: GenericChannelId,
    ) -> RedisResult<Option<IntelligenceMessages>> {
        let mut conn = self.conn.clone();
        let value: Option<IntelligenceMessages> = conn
            .get(keys::INTELLIGENCE_CONTEXT.user(user).channel(channel))
            .await?;

        Ok(value)
    }

    pub async fn set_intelligence_context(
        &self,
        user: UserId,
        channel: GenericChannelId,
        context: &IntelligenceMessages,
    ) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn
            .set_options(
                keys::INTELLIGENCE_CONTEXT.user(user).channel(channel),
                context,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(300)),
            )
            .await?;

        Ok(())
    }
}

impl Storage {
    pub async fn incr_warn_count(&self, user: UserId, guild: GuildId) -> RedisResult<u64> {
        let mut conn = self.conn.clone();
        let value: u64 = conn
            .incr(keys::WARN_COUNT.user(user).guild(guild), 1)
            .await?;

        Ok(value)
    }

    pub async fn del_warn_count(&self, user: UserId, guild: GuildId) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        () = conn.del(keys::WARN_COUNT.user(user).guild(guild)).await?;

        Ok(())
    }
}
