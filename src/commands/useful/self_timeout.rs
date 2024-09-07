use color_eyre::eyre::{eyre, Result};
use poise::serenity_prelude as serenity;

use crate::{utils::serenity::unique_username, Context};

/// Time yourself out for a specific duration
#[poise::command(rename = "self-timeout", slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn self_timeout(
    ctx: Context<'_>,
    #[description = "The duration to time yourself out for"] duration: String,
    #[description = "The reason for the timeout"] reason: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        if let Some(mut member) = ctx.author_member().await {
            let start = chrono::Utc::now();
            let end = start + duration;
            let end_serenity = serenity::Timestamp::from_unix_timestamp(end.timestamp())?;

            member
                .to_mut()
                .edit(
                    &ctx,
                    serenity::EditMember::default()
                        .disable_communication_until_datetime(end_serenity)
                        .audit_log_reason(&format!(
                            "Requested self timeout{}",
                            reason.as_ref().map_or(String::new(), |r| format!(": {r}"))
                        )),
                )
                .await?;

            let resp_embed = serenity::CreateEmbed::default()
                .title("Self-timeout in effect")
                .field("Start", format!("<t:{0}:F>", start.timestamp()), false)
                .field(
                    "End",
                    format!("<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                    false,
                )
                .color(0x4ade80);

            ctx.send(poise::CreateReply::default().embed(resp_embed.clone()))
                .await?;

            if let Some(storage) = &ctx.data().storage {
                if storage
                    .get_self_timeout_transparency(&ctx.author().id.to_string())
                    .await?
                    .unwrap_or(false)
                {
                    let mut resp_embed = resp_embed.author(
                        serenity::CreateEmbedAuthor::new(unique_username(ctx.author()))
                            .icon_url(ctx.author().face()),
                    );

                    if let Some(reason) = reason {
                        resp_embed = resp_embed.field("Reason", reason, false);
                    }

                    ctx.channel_id()
                        .send_message(&ctx, serenity::CreateMessage::default().embed(resp_embed))
                        .await?;
                };
            }
        } else {
            ctx.say("Error: Member unavailable!").await?;
        };
    } else {
        ctx.say("Error: Invalid duration!").await?;
    };

    Ok(())
}

/// Configure self-timeout transparency
#[poise::command(
    rename = "self-timeout-transparency",
    slash_command,
    guild_only,
    ephemeral
)]
pub async fn transparency(
    ctx: Context<'_>,
    #[description = "Whether transparency is on or off"] status: bool,
) -> Result<()> {
    let storage = ctx
        .data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available for the transparency feature"))?;

    storage
        .set_self_timeout_transparency(&ctx.author().id.to_string(), &status)
        .await?;

    let desc = if status {
        "Your self-timeouts will now be publicly logged to the channel that you ran the self-timeout in."
    } else {
        "Your self-timeouts will no longer be publicly logged."
    };

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Self-timeout transparency updated!")
                .description(desc)
                .color(0x4ade80),
        ),
    )
    .await?;

    Ok(())
}
