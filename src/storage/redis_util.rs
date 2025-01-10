#[macro_export]
macro_rules! impl_redis_serde {
    ($t: ty) => {
        impl ::redis::FromRedisValue for $t {
            fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                use ::redis::{ErrorKind, RedisError, Value};

                let Value::BulkString(bytes) = v else {
                    return Err(RedisError::from((
                        ErrorKind::TypeError,
                        "Expected a string",
                        format!("{v:?}"),
                    )));
                };

                ::serde_json::from_slice(&bytes).map_err(|e| {
                    RedisError::from((
                        ErrorKind::TypeError,
                        "Failed to deserialize JSON data",
                        format!("{e:?}"),
                    ))
                })
            }
        }

        impl ::redis::ToRedisArgs for $t {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis::RedisWrite,
            {
                out.write_arg(&::serde_json::to_vec(self).unwrap());
            }
        }
    };
}
