// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

pub mod log;
pub mod presence;

#[derive(Debug)]
pub struct Storage {
    redis: ::redis::Client,
}

impl From<redis::Client> for Storage {
    fn from(value: ::redis::Client) -> Self {
        Self { redis: value }
    }
}

impl Storage {
    pub async fn size(&self) -> ::redis::RedisResult<u64> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let keys: u64 = redis::cmd("DBSIZE").query_async(&mut conn).await?;
        Ok(keys)
    }
}

macro_rules! impl_storage {
    ($n: ident, $k: literal, $t: ty) => {
        paste::paste! {
            #[::tracing::instrument(skip(self))]
            pub async fn [<get_ $n>](&self) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let ret: Option<$t> = conn.get($k).await?;
                Ok(ret)
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<set_ $n>](&self, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.set($k, value).await?;
                Ok(())
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<del_ $n>](&self) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.del($k).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, $($mn: ident: $mt: ty),+) => {
        paste::paste! {
            #[::tracing::instrument(skip(self))]
            pub async fn [<get_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let ret: Option<$t> = conn.get(format!($k, $($mn),*)).await?;
                Ok(ret)
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<set_ $n>](&self, $($mn: $mt),+, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.set(format!($k, $($mn),*), value).await?;
                Ok(())
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<del_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.del(format!($k, $($mn),*)).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, ttl = $ttl: literal) => {
        paste::paste! {
            #[::tracing::instrument(skip(self))]
            pub async fn [<get_ $n>](&self) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let ret: Option<$t> = conn.get($k).await?;
                Ok(ret)
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<set_ $n>](&self, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                conn.set_options($k, value, redis::SetOptions::default().with_expiration(redis::SetExpiry::EX($ttl))).await?;
                Ok(())
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<del_ $n>](&self) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                conn.del($k).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, ttl = $ttl: literal, $($mn: ident: $mt: ty),+) => {
        paste::item! {
            #[::tracing::instrument(skip(self))]
            pub async fn [<get_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let ret: Option<$t> = conn.get(format!($k, $($mn),*)).await?;
                Ok(ret)
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<set_ $n>](&self, $($mn: $mt),+, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.set_options(format!($k, $($mn),*), value, redis::SetOptions::default().with_expiration(redis::SetExpiry::EX($ttl))).await?;
                Ok(())
            }

            #[::tracing::instrument(skip(self))]
            pub async fn [<del_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_multiplexed_async_connection().await?;
                let _: () = conn.del(format!($k, $($mn),*)).await?;
                Ok(())
            }
        }
    };
}

#[allow(dead_code)]
impl Storage {
    impl_storage!(presence, "presence-v1", presence::PresenceData);
    impl_storage!(starboard, "starboard-v1:{}", String, ttl = 2629746, message_id: &str);
    impl_storage!(self_timeout_transparency, "stt-v1:{}", bool, user_id: &str);
    impl_storage!(message_log, "message-log-v1:{}", log::MessageLog, ttl = 86400, message_id: &str);
}

impl Storage {
    #[tracing::instrument(skip(self))]
    pub async fn getall_autoreply(&self) -> ::redis::RedisResult<Vec<(String, String)>> {
        use ::redis::AsyncCommands as _;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let values: Vec<(String, String)> = conn.hgetall("autoreply-v1").await?;
        Ok(values)
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_autoreply(&self, f: &str, v: &str) -> ::redis::RedisResult<()> {
        use ::redis::AsyncCommands as _;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.hset("autoreply-v1", f, v).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn del_autoreply(&self, f: &str) -> ::redis::RedisResult<()> {
        use ::redis::AsyncCommands as _;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.hdel("autoreply-v1", f).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delall_autoreply(&self) -> ::redis::RedisResult<()> {
        use ::redis::AsyncCommands as _;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.del("autoreply-v1").await?;
        Ok(())
    }
}
