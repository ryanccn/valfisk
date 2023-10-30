use owo_colors::OwoColorize;
use poise::{serenity_prelude as serenity, CreateReply};
use redis::AsyncCommands;

use crate::Context;
use anyhow::Result;

#[derive(poise::ChoiceParameter, serde::Serialize, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PresenceChoice {
    Custom,
    Playing,
    Watching,
    Listening,
    Competing,
}

impl PresenceChoice {
    fn make_activity(&self, content: &str) -> serenity::ActivityData {
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

#[derive(serde::Serialize, serde::Deserialize)]
struct PresencePersistence {
    content: String,
    #[serde(rename = "type")]
    type_: PresenceChoice,
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

    ctx.serenity_context().set_presence(
        Some(type_.make_activity(&content)),
        serenity::OnlineStatus::Online,
    );

    ctx.send(
        CreateReply::new().embed(
            serenity::CreateEmbed::new()
                .title("Presence set!")
                .field("Type", type_.to_string(), false)
                .field("Content", &content, false)
                .color(0x4ade80),
        ),
    )
    .await?;

    if let Some(redis) = &ctx.data().redis {
        let mut conn = redis.get_async_connection().await?;
        conn.set(
            "presence-v1",
            serde_json::to_string(&PresencePersistence { content, type_ })?,
        )
        .await?;
    }

    Ok(())
}

pub async fn restore_presence(ctx: &serenity::Context, redis_client: &redis::Client) -> Result<()> {
    let mut conn = redis_client.get_async_connection().await?;
    let data: Option<String> = conn.get("presence-v1").await?;

    if let Some(data) = data {
        let data: PresencePersistence = serde_json::from_str(&data)?;
        ctx.set_presence(
            Some(data.type_.make_activity(&data.content)),
            serenity::OnlineStatus::Online,
        );
        println!("{} presence from Redis", "Restored".cyan());
    }

    Ok(())
}
