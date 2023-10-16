use anyhow::Result;

use crate::Context;

/// Pong! And also get the gateway heartbeat latency.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say("Pong").await?;

    Ok(())
}
