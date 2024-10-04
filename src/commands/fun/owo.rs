use eyre::Result;

use crate::Context;

/// owo
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn owo(ctx: Context<'_>) -> Result<()> {
    ctx.say("owo").await?;

    Ok(())
}
