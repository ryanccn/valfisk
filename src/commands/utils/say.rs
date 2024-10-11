use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Send a message in the current channel
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn say(
    ctx: Context<'_>,
    #[description = "Text to send in the current channel"] content: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    ctx.channel_id()
        .send_message(
            ctx.http(),
            serenity::CreateMessage::default().content(content),
        )
        .await?;

    ctx.say("Done!").await?;

    Ok(())
}
