// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use redis::{
    AsyncCommands as _, RedisResult,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use std::{fmt, time::Duration};

use log::MessageLog;
use reminder::ReminderData;

use crate::config::GuildConfig;

pub mod log;
pub mod presence;
mod redis_util;
pub mod reminder;

#[non_exhaustive]
pub struct Storage {
    conn: ConnectionManager,
}

impl fmt::Debug for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Storage").finish_non_exhaustive()
    }
}

impl Storage {
    pub async fn redis(redis: redis::Client) -> RedisResult<Self> {
        let conn = ConnectionManager::new_with_config(
            redis,
            ConnectionManagerConfig::new()
                .set_connection_timeout(Duration::from_secs(10))
                .set_response_timeout(Duration::from_secs(60))
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

mod consts {
    pub const GUILD_CONFIG: &str = "guild-config-v1";
    pub const PRESENCE: &str = "presence-v1";
    pub const STARBOARD: &str = "starboard-v2";
    pub const MESSAGE_LOG: &str = "message-log-v2";
    pub const REMINDERS: &str = "reminders-v1";
    pub const AUTOREPLY: &str = "autoreply-v2";
}

impl Storage {
    pub async fn get_config(&self, guild_id: u64) -> RedisResult<GuildConfig> {
        let mut conn = self.conn.clone();
        let ret: Option<GuildConfig> = conn
            .get(format!("{}:g{guild_id}", consts::GUILD_CONFIG))
            .await?;
        Ok(ret.unwrap_or_default())
    }

    pub async fn set_config(&self, guild_id: u64, value: &GuildConfig) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set(format!("{}:g{guild_id}", consts::GUILD_CONFIG), value)
            .await?;
        Ok(())
    }

    pub async fn del_config(&self, guild_id: u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .del(format!("{}:g{guild_id}", consts::GUILD_CONFIG))
            .await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_presence(&self) -> RedisResult<Option<presence::PresenceData>> {
        let mut conn = self.conn.clone();
        let ret: Option<presence::PresenceData> = conn.get(consts::PRESENCE).await?;
        Ok(ret)
    }

    pub async fn set_presence(&self, value: &presence::PresenceData) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn.set(consts::PRESENCE, value).await?;
        Ok(())
    }

    pub async fn del_presence(&self) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn.del(consts::PRESENCE).await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_starboard(&self, message_id: u64) -> RedisResult<Option<u64>> {
        let mut conn = self.conn.clone();
        let ret: Option<u64> = conn
            .get(format!("{}:m{message_id}", consts::STARBOARD))
            .await?;
        Ok(ret)
    }

    pub async fn set_starboard(&self, message_id: u64, value: &u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_options(
                format!("{}:m{message_id}", consts::STARBOARD),
                value,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(2592000)),
            )
            .await?;
        Ok(())
    }

    pub async fn del_starboard(&self, message_id: u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .del(format!("{}:m{message_id}", consts::STARBOARD))
            .await?;
        Ok(())
    }
}

impl Storage {
    pub async fn get_message_log(&self, message_id: u64) -> RedisResult<Option<MessageLog>> {
        let mut conn = self.conn.clone();
        let ret: Option<MessageLog> = conn
            .get(format!("{}:m{message_id}", consts::MESSAGE_LOG))
            .await?;
        Ok(ret)
    }

    pub async fn set_message_log(&self, message_id: u64, value: &MessageLog) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_options(
                format!("{}:m{message_id}", consts::MESSAGE_LOG),
                value,
                redis::SetOptions::default().with_expiration(redis::SetExpiry::EX(86400)),
            )
            .await?;
        Ok(())
    }

    pub async fn del_message_log(&self, message_id: u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .del(format!("{}:m{message_id}", consts::MESSAGE_LOG))
            .await?;
        Ok(())
    }
}

impl Storage {
    pub async fn scan_reminders(&self) -> RedisResult<Vec<ReminderData>> {
        use futures_util::StreamExt as _;

        let mut conn = self.conn.clone();
        let values: Vec<ReminderData> = redis::cmd("ZSCAN")
            .arg(consts::REMINDERS)
            .cursor_arg(0)
            .arg("NOSCORES")
            .clone()
            .iter_async::<ReminderData>(&mut conn)
            .await?
            .collect::<Vec<_>>()
            .await;

        Ok(values)
    }

    pub async fn add_reminders(&self, value: &ReminderData) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .zadd(consts::REMINDERS, value, value.timestamp.timestamp())
            .await?;

        Ok(())
    }

    pub async fn clean_reminders(&self) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .zrembyscore(consts::REMINDERS, 0, chrono::Utc::now().timestamp() - 1)
            .await?;

        Ok(())
    }
}

impl Storage {
    pub async fn scan_autoreply(&self, guild_id: u64) -> RedisResult<Vec<(String, String)>> {
        use futures_util::StreamExt as _;

        let mut conn = self.conn.clone();
        let values: Vec<(String, String)> = conn
            .hscan(format!("{}:g{guild_id}", consts::AUTOREPLY))
            .await?
            .collect::<Vec<_>>()
            .await;

        Ok(values)
    }

    pub async fn add_autoreply(&self, guild_id: u64, f: &str, v: &str) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .hset(format!("{}:g{guild_id}", consts::AUTOREPLY), f, v)
            .await?;
        Ok(())
    }

    pub async fn del_autoreply(&self, guild_id: u64, f: &str) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .hdel(format!("{}:g{guild_id}", consts::AUTOREPLY), f)
            .await?;
        Ok(())
    }

    pub async fn delall_autoreply(&self, guild_id: u64) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .del(format!("{}:g{guild_id}", consts::AUTOREPLY))
            .await?;
        Ok(())
    }
}
