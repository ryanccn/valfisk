use eyre::Result;
use poise::serenity_prelude as serenity;

use super::LOGS_CHANNEL;
use crate::Context;

/// Timeout a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "The member to timeout"] mut member: serenity::Member,
    #[description = "Duration of timeout"] duration: String,
    #[description = "Reason for the timeout"] reason: Option<String>,
    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        let end = chrono::Utc::now() + duration;
        let end_serenity = serenity::Timestamp::from_unix_timestamp(end.timestamp())?;

        let mut edit_member =
            serenity::EditMember::default().disable_communication_until(end_serenity);

        if let Some(reason) = &reason {
            edit_member = edit_member.audit_log_reason(reason);
        }

        member.edit(ctx.http(), edit_member).await?;

        let mut dm_embed = serenity::CreateEmbed::default()
            .title("Timeout")
            .field("User", format!("{} ({})", member, member.user.id), false)
            .color(0x9775fa)
            .timestamp(serenity::Timestamp::now());

        if let Some(reason) = &reason {
            dm_embed = dm_embed.field("Reason", reason, false);
        }

        dm_embed = dm_embed.field(
            "Duration",
            humantime::format_duration(duration).to_string(),
            false,
        );

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
    } else {
        ctx.say("Invalid duration provided!").await?;
    }

    Ok(())
}
