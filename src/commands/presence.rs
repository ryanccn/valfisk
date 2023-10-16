use anyhow::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::Context;

#[derive(poise::ChoiceParameter)]
pub enum PresenceChoice {
    Playing,
    Watching,
    Custom,
}

impl std::fmt::Display for PresenceChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PresenceChoice::Playing => "Playing".to_owned(),
                PresenceChoice::Watching => "Watching".to_owned(),
                PresenceChoice::Custom => "Custom".to_owned(),
            }
        )
    }
}

fn make_activity(content: &str, presence_type: &PresenceChoice) -> serenity::ActivityData {
    match presence_type {
        PresenceChoice::Playing => serenity::ActivityData::playing(content),
        PresenceChoice::Watching => serenity::ActivityData::watching(content),
        PresenceChoice::Custom => serenity::ActivityData::custom(content),
    }
}

/// Modify the Discord presence shown by the bot
#[poise::command(slash_command, ephemeral)]
pub async fn presence(
    ctx: Context<'_>,
    #[description = "Text to display"] content: String,

    #[rename = "type"]
    #[description = "Type of presence"]
    type_: PresenceChoice,
) -> Result<()> {
    ctx.serenity_context().set_presence(
        Some(make_activity(&content, &type_)),
        serenity::OnlineStatus::Online,
    );

    ctx.send(
        CreateReply::new().embed(
            serenity::CreateEmbed::new()
                .title("Presence set!")
                .description(format!("Presence set to **{} {}**", &type_, &content))
                .color(0x4ade80),
        ),
    )
    .await?;

    Ok(())
}
