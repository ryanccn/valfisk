use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Time yourself out for a specific duration
#[poise::command(rename = "self-timeout", slash_command, guild_only)]
pub async fn self_timeout(
    ctx: Context<'_>,
    #[description = "The duration to time yourself out for"] duration: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        if let Some(mut member) = ctx.author_member().await {
            let start = chrono::Utc::now();
            let end = start + duration;
            let end_serenity = serenity::Timestamp::from_unix_timestamp(end.timestamp())?;

            member
                .to_mut()
                .disable_communication_until(&ctx, end_serenity)
                .await?;

            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Self-timeout in effect")
                        .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                        .field(
                            "End",
                            format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                            false,
                        )
                        .color(0x4ade80),
                ),
            )
            .await?;
        } else {
            ctx.say("Error: Member unavailable!").await?;
        };
    } else {
        ctx.say("Error: Invalid duration!").await?;
    };

    Ok(())
}
