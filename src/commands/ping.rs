use color_eyre::eyre::Result;

use crate::Context;

/// Pong!
#[poise::command(slash_command, guild_only)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say("Pong!").await?;

    Ok(())
}
