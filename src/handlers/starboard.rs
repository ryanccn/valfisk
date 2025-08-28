// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::str::FromStr;

use eyre::{Result, eyre};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::config::GuildConfig;

async fn get_starboard_channel(
    ctx: &serenity::Context,
    guild_config: &GuildConfig,
    channel: serenity::GenericChannelId,
    guild: Option<serenity::GuildId>,
) -> Result<Option<serenity::GenericChannelId>> {
    let Some(channel) = channel
        .to_channel(&ctx, guild)
        .await
        .ok()
        .and_then(|ch| ch.guild())
    else {
        return Ok(None);
    };

    if guild_config.private_category.is_some()
        && guild_config.private_category == channel.parent_id.map(|id| id.widen())
    {
        return Ok(guild_config.private_starboard_channel);
    }

    let guild = channel.base.guild_id.to_partial_guild(ctx).await?;

    let everyone_role = guild
        .roles
        .get(&guild.id.everyone_role())
        .ok_or_else(|| eyre!("could not obtain @everyone role"))?;

    let guild_category = match &channel.parent_id {
        Some(parent_id) => Some(parent_id.to_guild_channel(&ctx, Some(guild.id)).await?),
        None => None,
    };

    if channel.permission_overwrites.iter().any(|p| {
        p.kind == serenity::PermissionOverwriteType::Role(everyone_role.id)
            && (p.allow.view_channel())
    }) || !channel.permission_overwrites.iter().any(|p| {
        p.kind == serenity::PermissionOverwriteType::Role(everyone_role.id)
            && (p.deny.view_channel())
    }) && !guild_category.as_ref().is_some_and(|cat| {
        cat.permission_overwrites.iter().any(|p| {
            p.kind == serenity::PermissionOverwriteType::Role(everyone_role.id)
                && (p.deny.view_channel())
        })
    }) && everyone_role.permissions.view_channel()
    {
        return Ok(guild_config.starboard_channel);
    }

    Ok(None)
}

#[derive(Default, Debug)]
pub enum StarboardEmojis {
    Any,
    Allowlist(Vec<String>),
    #[default]
    None,
}

impl StarboardEmojis {
    pub fn allow(&self, reaction: &serenity::MessageReaction) -> bool {
        match self {
            Self::Any => true,
            Self::Allowlist(list) => match &reaction.reaction_type {
                serenity::ReactionType::Custom { id, .. } => list.contains(&id.to_string()),
                serenity::ReactionType::Unicode(fixed_string) => {
                    list.contains(&fixed_string.to_string())
                }
                _ => false,
            },
            Self::None => false,
        }
    }
}

impl FromStr for StarboardEmojis {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();

        if s == "*" {
            Ok(Self::Any)
        } else if s.is_empty() {
            Ok(Self::None)
        } else {
            Ok(Self::Allowlist(
                s.split(',')
                    .map(|c| c.trim().to_owned())
                    .collect::<Vec<_>>(),
            ))
        }
    }
}

impl<'de> serde::Deserialize<'de> for StarboardEmojis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

fn is_significant_reaction(
    guild_config: &GuildConfig,
    reaction: &serenity::MessageReaction,
) -> bool {
    guild_config
        .starboard_emojis
        .as_deref()
        .unwrap_or_default()
        .parse::<StarboardEmojis>()
        .is_ok_and(|r| r.allow(reaction))
        && reaction.count >= guild_config.starboard_threshold.unwrap_or(3)
}

fn get_significant_reactions<'a>(
    guild_config: &GuildConfig,
    message: &'a serenity::Message,
) -> Vec<(&'a serenity::ReactionType, u64)> {
    let mut collected_reactions: Vec<(&serenity::ReactionType, u64)> = message
        .reactions
        .iter()
        .filter(|r| is_significant_reaction(guild_config, r))
        .map(|r| (&r.reaction_type, r.count))
        .collect();

    collected_reactions.sort_by_key(|i| match &i.0 {
        serenity::ReactionType::Custom { id, .. } => id.get().to_string(),
        serenity::ReactionType::Unicode(str) => str.to_string(),
        _ => "unknown".to_owned(),
    });

    collected_reactions
}

fn serialize_reactions(
    channel: serenity::GenericChannelId,
    reactions: &[(&serenity::ReactionType, u64)],
) -> String {
    let reaction_string = reactions
        .iter()
        .map(|r| format!("{} {}", r.0, r.1))
        .collect::<Vec<_>>()
        .join(" ");

    format!("**{reaction_string}** in {}", channel.mention())
}

async fn make_message_embed<'a>(
    ctx: &'a serenity::Context,
    message: &'a serenity::Message,
) -> serenity::CreateEmbed<'a> {
    let content = message.content.to_string();
    let mut builder = serenity::CreateEmbed::default()
        .description(if content.is_empty() {
            "*No content*".to_owned()
        } else {
            content
        })
        .author(
            serenity::CreateEmbedAuthor::new(message.author.tag()).icon_url(message.author.face()),
        )
        .timestamp(message.timestamp);

    if let Some(reference) = &message.message_reference
        && let Some(message_id) = reference.message_id {
            builder = builder.field(
                "Replying to",
                message_id.link(reference.channel_id, reference.guild_id),
                false,
            );
        }

    if let Some(image_attachment) = message.attachments.iter().find(|att| {
        att.content_type
            .as_ref()
            .is_some_and(|ct| ct.starts_with("image/"))
    }) {
        builder = builder.image(image_attachment.url.to_string());
    }

    builder = builder.color(0xffd43b);

    if let Some(guild_id) = message
        .guild_channel(&ctx)
        .await
        .ok()
        .map(|ch| ch.base.guild_id)
        && let Ok(member) = guild_id.member(&ctx, message.author.id).await
            && let Some(mut roles) = member.roles(&ctx.cache) {
                roles.retain(|r| r.colour.0 != 0);
                roles.sort_unstable_by_key(|r| r.position);

                if let Some(role) = roles.last() {
                    builder = builder.color(role.colour);
                }
            }

    builder
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(
    ctx: &serenity::Context,
    guild_id: Option<serenity::GuildId>,
    message: &serenity::Message,
) -> Result<()> {
    if let Some(guild_id) = guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage {
            let guild_config = storage.get_config(guild_id).await?;

            if let Some(starboard) =
                get_starboard_channel(ctx, &guild_config, message.channel_id, message.guild_id)
                    .await?
            {
                let significant_reactions = get_significant_reactions(&guild_config, message);

                if let Some(existing_starboard_message) =
                    storage.get_starboard(message.id).await?.map(|s| s.into())
                {
                    if significant_reactions.is_empty() {
                        storage.del_starboard(message.id).await?;

                        let _ = starboard
                            .delete_message(&ctx.http, existing_starboard_message, None)
                            .await;

                        tracing::debug!(
                            starboard_id = existing_starboard_message.get(),
                            message_id = message.id.get(),
                            "deleted starboard message"
                        );
                    } else {
                        starboard
                            .edit_message(
                                &ctx.http,
                                existing_starboard_message,
                                serenity::EditMessage::default().content(serialize_reactions(
                                    message.channel_id,
                                    &significant_reactions,
                                )),
                            )
                            .await?;

                        tracing::debug!(
                            starboard_id = existing_starboard_message.get(),
                            message_id = message.id.get(),
                            "edited starboard message"
                        );
                    }
                } else if !significant_reactions.is_empty() {
                    let content = serialize_reactions(message.channel_id, &significant_reactions);
                    let embed = make_message_embed(ctx, message).await;

                    let row = serenity::CreateActionRow::Buttons(
                        vec![
                            serenity::CreateButton::new_link(message.link())
                                .label("Jump to message"),
                        ]
                        .into(),
                    );

                    let starboard_message = starboard
                        .send_message(
                            &ctx.http,
                            serenity::CreateMessage::default()
                                .content(content)
                                .embed(embed)
                                .components(vec![serenity::CreateComponent::ActionRow(row)]),
                        )
                        .await?;

                    storage
                        .set_starboard(message.id, &starboard_message.id.get())
                        .await?;

                    tracing::debug!(
                        starboard_id = starboard_message.id.get(),
                        message_id = message.id.get(),
                        "created starboard message"
                    );
                }
            }
        }

    Ok(())
}

#[tracing::instrument(skip(ctx))]
pub async fn handle_deletion(
    ctx: &serenity::Context,
    deleted_message_id: serenity::MessageId,
    channel_id: serenity::GenericChannelId,
    guild_id: Option<serenity::GuildId>,
) -> Result<()> {
    if let Some(guild_id) = guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage {
            let guild_config = storage.get_config(guild_id).await?;

            if let Some(starboard_channel) =
                get_starboard_channel(ctx, &guild_config, channel_id, Some(guild_id)).await?
                && let Some(starboard_id) = storage.get_starboard(deleted_message_id).await? {
                    storage.del_starboard(deleted_message_id).await?;

                    let _ = ctx
                        .http
                        .delete_message(starboard_channel, starboard_id.into(), None)
                        .await;

                    tracing::debug!(
                        starboard_id,
                        message_id = deleted_message_id.get(),
                        "deleted starboard message (source deleted)",
                    );
                }
        }

    Ok(())
}
