// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::{
    CreateReply,
    serenity_prelude::{self as serenity, Mentionable as _},
};

use crate::{Context, utils};

async fn recreate(
    ctx: Context<'_>,
    channel: serenity::GenericChannelId,
    guild: serenity::GuildId,
    actor: &serenity::User,
) -> Result<Option<serenity::GenericChannelId>> {
    let audit_log_reason = format!("Log rotation by {} ({})", actor.tag(), actor.id);

    let Some(channel) = channel
        .to_channel(&ctx, Some(guild))
        .await
        .ok()
        .and_then(|ch| ch.guild())
    else {
        return Ok(None);
    };

    let mut create_channel = serenity::CreateChannel::new(&channel.base.name)
        .kind(serenity::ChannelType::Text)
        .nsfw(channel.nsfw)
        .permissions(&channel.permission_overwrites)
        .position(channel.position)
        .audit_log_reason(&audit_log_reason);

    if let Some(data) = channel.parent_id {
        create_channel = create_channel.category(data);
    }

    if let Some(data) = channel.default_auto_archive_duration {
        create_channel = create_channel.default_auto_archive_duration(data);
    }

    let new_channel = guild.create_channel(ctx.http(), create_channel).await?;

    channel.delete(ctx.http(), Some(&audit_log_reason)).await?;

    Ok(Some(new_channel.id.widen()))
}

#[derive(poise::ChoiceParameter, PartialEq, Eq, Clone, Copy, Debug)]
enum RotateLogsKind {
    Moderation,
    Message,
    Member,
}

/// Rotate logs channels by recreating and automatically configuring them
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    rename = "rotate-logs",
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn rotate_logs(
    ctx: Context<'_>,
    #[description = "Only rotate one kind of logs channel"] kind: Option<RotateLogsKind>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let actor = ctx.author();

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let (confirmed, reply) = utils::serenity::interaction_confirm(
        &ctx,
        serenity::CreateEmbed::default()
            .title("Rotate logs")
            .description("Are you sure you want to rotate logs channels? This will delete the configured logs channels and create new ones.")
            .color(0xffd43b),
    )
    .await?;

    if confirmed {
        let mut guild_config = storage.get_config(guild_id).await?;
        let mut new_channels = Vec::new();

        if kind.is_none_or(|k| k == RotateLogsKind::Moderation)
            && let Some(channel) = guild_config.moderation_logs_channel
            && let Some(ch) = recreate(ctx, channel, guild_id, actor).await?
        {
            new_channels.push(ch);
            guild_config.moderation_logs_channel = Some(ch);
        }

        if kind.is_none_or(|k| k == RotateLogsKind::Message)
            && let Some(channel) = guild_config.message_logs_channel
            && let Some(ch) = recreate(ctx, channel, guild_id, actor).await?
        {
            new_channels.push(ch);
            guild_config.message_logs_channel = Some(ch);
        }

        if kind.is_none_or(|k| k == RotateLogsKind::Member)
            && let Some(channel) = guild_config.member_logs_channel
            && let Some(ch) = recreate(ctx, channel, guild_id, actor).await?
        {
            new_channels.push(ch);
            guild_config.member_logs_channel = Some(ch);
        }

        storage.set_config(guild_id, &guild_config).await?;

        reply
            .edit(
                ctx,
                CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .title("Rotated logs")
                            .description(
                                new_channels
                                    .iter()
                                    .map(|ch| ch.mention().to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            )
                            .color(0x4ade80),
                    )
                    .components(vec![]),
            )
            .await?;
    } else {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .title("Log rotation cancelled")
                            .color(0xff6b6b),
                    )
                    .components(vec![]),
            )
            .await?;
    }

    Ok(())
}
