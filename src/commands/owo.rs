use anyhow::Result;

use crate::Context;

/// owo
#[poise::command(slash_command, guild_only)]
pub async fn owo(ctx: Context<'_>) -> Result<()> {
    ctx.say("owo").await?;

    Ok(())
}
