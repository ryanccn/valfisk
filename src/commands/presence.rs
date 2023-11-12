use crate::Context;
use poise::{serenity_prelude as serenity, CreateReply};

use redis::AsyncCommands;
use redis_macros::{FromRedisValue, ToRedisArgs};

use anyhow::Result;
use owo_colors::OwoColorize;

#[derive(poise::ChoiceParameter, serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PresenceChoice {
    Custom,
    Playing,
    Watching,
    Listening,
    Competing,
}

impl PresenceChoice {
    fn make_activity(self, content: &str) -> serenity::ActivityData {
        match self {
            Self::Custom => serenity::ActivityData::custom(content),
            Self::Playing => serenity::ActivityData::playing(content),
            Self::Watching => serenity::ActivityData::watching(content),
            Self::Listening => serenity::ActivityData::listening(content),
            Self::Competing => serenity::ActivityData::competing(content),
        }
    }
}

impl Default for PresenceChoice {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for PresenceChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Custom => "Custom".to_owned(),
                Self::Playing => "Playing".to_owned(),
                Self::Watching => "Watching".to_owned(),
                Self::Listening => "Listening".to_owned(),
                Self::Competing => "Competing".to_owned(),
            }
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, FromRedisValue, ToRedisArgs, Clone, Debug)]
struct PresenceData {
    content: String,
    #[serde(rename = "type")]
    type_: PresenceChoice,
}

impl PresenceData {
    fn make_activity(&self) -> serenity::ActivityData {
        self.type_.make_activity(&self.content)
    }
}

/// Modify the Discord presence shown by the bot
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn presence(
    ctx: Context<'_>,
    #[description = "Text to display"] content: String,

    #[rename = "type"]
    #[description = "Type of presence"]
    type_: Option<PresenceChoice>,
) -> Result<()> {
    let type_ = type_.unwrap_or_default();
    let presence_data = PresenceData { content, type_ };

    ctx.serenity_context().set_presence(
        Some(presence_data.make_activity()),
        serenity::OnlineStatus::Online,
    );

    ctx.send(
        CreateReply::new().embed(
            serenity::CreateEmbed::new()
                .title("Presence set!")
                .field("Type", presence_data.type_.to_string(), false)
                .field("Content", &presence_data.content, false)
                .color(0x4ade80),
        ),
    )
    .await?;

    if let Some(redis) = &ctx.data().redis {
        let mut conn = redis.get_async_connection().await?;
        conn.set("presence-v1", presence_data).await?;
    }

    Ok(())
}

pub async fn restore(ctx: &serenity::Context, redis_client: &redis::Client) -> Result<()> {
    let mut conn = redis_client.get_async_connection().await?;
    let data: Option<PresenceData> = conn.get("presence-v1").await?;

    if let Some(data) = data {
        ctx.set_presence(Some(data.make_activity()), serenity::OnlineStatus::Online);
        println!("{} presence from Redis", "Restored".cyan());
    }

    Ok(())
}
