// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::{config::GuildConfig, handlers::intelligence::IntelligenceMessages};

use super::{log::MessageLog, presence::PresenceData, reminder::ReminderData};

macro_rules! impl_redis_serde {
    ($($t: ty),+ $(,)?) => {
        $(
        impl ::redis::FromRedisValue for $t {
            fn from_redis_value(v: ::redis::Value) -> Result<Self, ::redis::ParsingError> {
                use ::redis::{ParsingError, Value};

                let Value::BulkString(bytes) = v else {
                    return Err(ParsingError::from(format!("Expected a string: {v:?}")));
                };

                ::serde_json::from_slice(&bytes).map_err(|e| {
                    ParsingError::from(format!("Failed to deserialize JSON data: {e:?}"))
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

        impl ::redis::ToSingleRedisArg for $t {}
        )+
    };
}

impl_redis_serde!(
    PresenceData,
    ReminderData,
    MessageLog,
    GuildConfig,
    IntelligenceMessages
);
