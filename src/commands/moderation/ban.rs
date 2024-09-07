use color_eyre::eyre::Result;
use poise::serenity_prelude as serenity;

use super::LOGS_CHANNEL;
use crate::{utils::serenity::unique_username, Context};

/// Ban a member
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::Member,
    #[description = "Reason for the ban"] reason: Option<String>,

    #[description = "Days of messages to delete (default: 0)"]
    #[min = 0]
    #[max = 7]
    delete_message_days: Option<u8>,

    #[description = "Notify with a direct message (default: true)"] dm: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Some(reason) = &reason {
        member.ban_with_reason(&ctx, 0, reason).await?;
    } else {
        member.ban(&ctx, 0).await?;
    }

    let mut dm_embed = serenity::CreateEmbed::default()
        .title("Ban")
        .field("User", format!("{} ({})", member, member.user.id), false)
        .color(0xda77f2)
        .timestamp(serenity::Timestamp::now());

    if let Some(reason) = &reason {
        dm_embed = dm_embed.field("Reason", reason, false);
    }

    dm_embed = dm_embed.field(
        "Days of messages deleted",
        delete_message_days.unwrap_or(0).to_string(),
        false,
    );

    if dm.unwrap_or(true) {
        member
            .user
            .direct_message(
                &ctx,
                serenity::CreateMessage::default().embed(dm_embed.clone()),
            )
            .await?;
    }

    if let Some(logs_channel) = *LOGS_CHANNEL {
        let server_embed = dm_embed.footer(
            serenity::CreateEmbedFooter::new(unique_username(ctx.author()))
                .icon_url(ctx.author().face()),
        );

        logs_channel
            .send_message(&ctx, serenity::CreateMessage::default().embed(server_embed))
            .await?;
    }

    ctx.say("Success!").await?;

    Ok(())
}
