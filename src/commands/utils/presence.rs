use crate::{storage::presence::PresenceChoice, Context};
use poise::{serenity_prelude as serenity, CreateReply};

use color_eyre::eyre::Result;
use tracing::info;

/// Modify the Discord presence shown by the bot
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MANAGE_GUILD"
)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn presence(
    ctx: Context<'_>,
    #[description = "Text to display"] content: String,
    #[description = "Type of presence"] r#type: Option<PresenceChoice>,
) -> Result<()> {
    let data = r#type.unwrap_or_default().to_data(&content);

    ctx.serenity_context()
        .set_presence(Some(data.to_activity()), serenity::OnlineStatus::Online);

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Presence set!")
                .field("Type", data.r#type.to_string(), false)
                .field("Content", &data.content, false)
                .color(0x4ade80),
        ),
    )
    .await?;

    if let Some(storage) = &ctx.data().storage {
        storage.set_presence(&data).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(ctx))]
pub async fn restore(ctx: &serenity::Context, storage: &crate::storage::Storage) -> Result<()> {
    let data = storage.get_presence().await?;

    if let Some(data) = data {
        ctx.set_presence(Some(data.to_activity()), serenity::OnlineStatus::Online);
        info!("Restored presence from Redis");
    }

    Ok(())
}
