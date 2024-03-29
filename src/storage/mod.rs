pub mod presence;

pub struct Storage {
    redis: ::redis::Client,
}

impl From<redis::Client> for Storage {
    fn from(value: ::redis::Client) -> Self {
        Storage { redis: value }
    }
}

impl Storage {
    pub async fn size(&self) -> ::redis::RedisResult<u64> {
        let mut conn = self.redis.get_async_connection().await?;
        let keys: u64 = redis::cmd("DBSIZE").query_async(&mut conn).await?;
        Ok(keys)
    }
}

macro_rules! impl_storage {
    ($n: ident, $k: literal, $t: ty) => {
        paste::paste! {
            pub async fn [<get_ $n>](&self) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                let ret: Option<$t> = conn.get($k).await?;
                Ok(ret)
            }

            pub async fn [<set_ $n>](&self, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.set($k, value).await?;
                Ok(())
            }

            pub async fn [<del_ $n>](&self) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.del($k).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, $($mn: ident: $mt: ty),+) => {
        paste::paste! {
            pub async fn [<get_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                let ret: Option<$t> = conn.get(format!($k, $($mn),*)).await?;
                Ok(ret)
            }

            pub async fn [<set_ $n>](&self, $($mn: $mt),+, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.set(format!($k, $($mn),*), value).await?;
                Ok(())
            }

            pub async fn [<del_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.del(format!($k, $($mn),*)).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, ttl = $ttl: literal) => {
        paste::paste! {
            pub async fn [<get_ $n>](&self) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                let ret: Option<$t> = conn.get($k).await?;
                Ok(ret)
            }

            pub async fn [<set_ $n>](&self, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.set_options($k, value, redis::SetOptions::default().with_expiration(redis::SetExpiry::EX($ttl))).await?;
                Ok(())
            }

            pub async fn [<del_ $n>](&self) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.del($k).await?;
                Ok(())
            }
        }
    };

    ($n: ident, $k: literal, $t: ty, ttl = $ttl: literal, $($mn: ident: $mt: ty),+) => {
        paste::item! {
            pub async fn [<get_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<Option<$t>> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                let ret: Option<$t> = conn.get(format!($k, $($mn),*)).await?;
                Ok(ret)
            }

            pub async fn [<set_ $n>](&self, $($mn: $mt),+, value: &$t) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.set_options(format!($k, $($mn),*), value, redis::SetOptions::default().with_expiration(redis::SetExpiry::EX($ttl))).await?;
                Ok(())
            }

            pub async fn [<del_ $n>](&self, $($mn: $mt),+) -> ::redis::RedisResult<()> {
                use ::redis::AsyncCommands as _;
                let mut conn = self.redis.get_async_connection().await?;
                conn.del(format!($k, $($mn),*)).await?;
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
}
