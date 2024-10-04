use eyre::Result;
use poise::serenity_prelude as serenity;

use super::LOGS_CHANNEL;
use crate::Context;

/// Kick a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The member to kick"] member: serenity::Member,
    #[description = "Reason for the kick"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    member.kick(ctx.http(), reason.as_deref()).await?;

    let mut dm_embed = serenity::CreateEmbed::default()
        .title("Kick")
        .field("User", format!("{} ({})", member, member.user.id), false)
        .color(0xf783ac)
        .timestamp(serenity::Timestamp::now());

    if let Some(reason) = &reason {
        dm_embed = dm_embed.field("Reason", reason, false);
    }

    if dm.unwrap_or(true) {
        if member
            .user
            .direct_message(
                ctx.http(),
                serenity::CreateMessage::default().embed(dm_embed.clone()),
            )
            .await
            .is_ok()
        {
            dm_embed = dm_embed.field("User notified", "Yes", false);
        } else {
            dm_embed = dm_embed.field("User notified", "Failed", false);
        }
    } else {
        dm_embed = dm_embed.field("User notified", "No", false);
    }

    if let Some(logs_channel) = *LOGS_CHANNEL {
        let server_embed = dm_embed.footer(
            serenity::CreateEmbedFooter::new(ctx.author().tag()).icon_url(ctx.author().face()),
        );

        logs_channel
            .send_message(
                ctx.http(),
                serenity::CreateMessage::default().embed(server_embed),
            )
            .await?;
    }

    ctx.say("Success!").await?;

    Ok(())
}