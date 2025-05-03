// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::{collections::HashSet, hash::Hash, str::FromStr};

use eyre::{Result, eyre};
use poise::{
    CreateReply,
    serenity_prelude::{CreateEmbed, GenericChannelId, RoleId, Timestamp},
};

use crate::Context;

#[derive(poise::ChoiceParameter, Clone, Debug)]
enum GuildConfigKey {
    #[name = "private_category"]
    PrivateCategory,
    #[name = "private_starboard_channel"]
    PrivateStarboardChannel,
    #[name = "starboard_channel"]
    StarboardChannel,
    #[name = "starboard_emojis"]
    StarboardEmojis,
    #[name = "starboard_threshold"]
    StarboardThreshold,
    #[name = "moderation_logs_channel"]
    ModerationLogsChannel,
    #[name = "message_logs_channel"]
    MessageLogsChannel,
    #[name = "member_logs_channel"]
    MemberLogsChannel,
    #[name = "moderator_role"]
    ModeratorRole,
    #[name = "logs_excluded_channels"]
    LogsExcludedChannels,
    #[name = "random_color_roles"]
    RandomColorRoles,
}

fn parse_id_set<T>(s: &str) -> Result<HashSet<T>>
where
    T: FromStr + Eq + Hash,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    s.split([','])
        .map(|f| f.trim().parse::<T>())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|err| err.into())
}

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("get", "put", "del", "reset"),
    subcommand_required,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn config(ctx: Context<'_>) -> Result<()> {
    Ok(())
}

/// Get guild configuration
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn get(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let data = storage.get_config(guild_id).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Configuration")
                .description(format!(
                    "```json\n{}\n```",
                    serde_json::to_string_pretty(&data)?
                ))
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

/// Set a key on the guild configuration
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
#[expect(clippy::too_many_arguments)]
async fn put(
    ctx: Context<'_>,

    #[description = "Private category that uses a separate starboard"]
    #[channel_types("Category")]
    private_category: Option<GenericChannelId>,

    #[description = "Separate starboard channel to use for the private category"]
    #[channel_types("Text")]
    private_starboard_channel: Option<GenericChannelId>,

    #[description = "Starboard channel to use for channels viewable by @everyone"]
    #[channel_types("Text")]
    starboard_channel: Option<GenericChannelId>,

    #[description = "Comma separated list of starboard emojis; `*` matches all emojis"]
    starboard_emojis: Option<String>,

    #[description = "Threshold of reactions for messages to be shown on the starboard"]
    starboard_threshold: Option<u64>,

    #[description = "Channel for moderation logs (e.g. bans)"]
    #[channel_types("Text")]
    moderation_logs_channel: Option<GenericChannelId>,

    #[description = "Channel for message logs (e.g. edits)"]
    #[channel_types("Text")]
    message_logs_channel: Option<GenericChannelId>,

    #[description = "Channel for member logs (e.g. joins)"]
    #[channel_types("Text")]
    member_logs_channel: Option<GenericChannelId>,

    #[description = "Role that moderators are assigned to, used for mentions"]
    moderator_role: Option<RoleId>,

    #[description = "Comma separated list of channels excluded from message logs"]
    logs_excluded_channels: Option<String>,

    #[description = "Comma separated list of roles to rotate colors for"]
    random_color_roles: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let mut data = storage.get_config(guild_id).await?;

    if let Some(value) = private_category {
        data.private_category = Some(value);
    }

    if let Some(value) = private_starboard_channel {
        data.private_starboard_channel = Some(value);
    }

    if let Some(value) = starboard_channel {
        data.starboard_channel = Some(value);
    }

    if let Some(value) = starboard_emojis {
        data.starboard_emojis = Some(value);
    }

    if let Some(value) = starboard_threshold {
        data.starboard_threshold = Some(value);
    }

    if let Some(value) = moderation_logs_channel {
        data.moderation_logs_channel = Some(value);
    }

    if let Some(value) = message_logs_channel {
        data.message_logs_channel = Some(value);
    }

    if let Some(value) = member_logs_channel {
        data.member_logs_channel = Some(value);
    }

    if let Some(value) = moderator_role {
        data.moderator_role = Some(value);
    }

    if let Some(value) = logs_excluded_channels {
        data.logs_excluded_channels = parse_id_set(&value)?;
    }

    if let Some(value) = random_color_roles {
        data.random_color_roles = parse_id_set(&value)?;
    }

    storage.set_config(guild_id, &data).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Updated configuration")
                .description(format!(
                    "```json\n{}\n```",
                    serde_json::to_string_pretty(&data)?
                ))
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

/// Delete a key on the guild configuration
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn del(
    ctx: Context<'_>,
    #[description = "The configuration key to delete"] key: GuildConfigKey,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let mut data = storage.get_config(guild_id).await?;

    match key {
        GuildConfigKey::PrivateCategory => data.private_category = None,
        GuildConfigKey::PrivateStarboardChannel => {
            data.private_starboard_channel = None;
        }
        GuildConfigKey::StarboardChannel => data.starboard_channel = None,
        GuildConfigKey::StarboardEmojis => data.starboard_emojis = None,
        GuildConfigKey::StarboardThreshold => {
            data.starboard_threshold = None;
        }
        GuildConfigKey::ModerationLogsChannel => {
            data.moderation_logs_channel = None;
        }
        GuildConfigKey::MessageLogsChannel => data.message_logs_channel = None,
        GuildConfigKey::MemberLogsChannel => data.member_logs_channel = None,
        GuildConfigKey::ModeratorRole => data.moderator_role = None,
        GuildConfigKey::LogsExcludedChannels => data.logs_excluded_channels = HashSet::new(),
        GuildConfigKey::RandomColorRoles => data.random_color_roles = HashSet::new(),
    }

    storage.set_config(guild_id, &data).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Deleted configuration key")
                .description(format!(
                    "```json\n{}\n```",
                    serde_json::to_string_pretty(&data)?
                ))
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

/// Reset the guild configuration to defaults
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn reset(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    storage.del_config(guild_id).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Reset configuration")
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}
