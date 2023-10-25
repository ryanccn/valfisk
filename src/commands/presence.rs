use anyhow::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::Context;

#[derive(poise::ChoiceParameter)]
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
                .field("Content", content, false)
                .color(0x4ade80),
        ),
    )
    .await?;

    Ok(())
}
