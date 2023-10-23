use anyhow::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Send a message in the current channel
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
pub async fn say(
    ctx: Context<'_>,
    #[description = "Text to send in the current channel"] content: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let channel = ctx.channel_id();

    channel
        .send_message(&ctx, serenity::CreateMessage::new().content(content))
        .await?;

    ctx.say("Done!").await?;

    Ok(())
}
