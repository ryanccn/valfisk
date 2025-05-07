// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::borrow::Cow;

use poise::serenity_prelude as serenity;

use eyre::Result;

use crate::{config::GuildConfig, storage::log::MessageLog, utils};

#[derive(Debug, Clone, Copy)]
pub struct LogMessageIds {
    pub message: serenity::MessageId,
    pub channel: serenity::GenericChannelId,
    pub guild: Option<serenity::GuildId>,
    pub author: Option<serenity::UserId>,
}

impl LogMessageIds {
    fn link(&self) -> String {
        self.message.link(self.channel, self.guild)
    }
}

impl From<&serenity::Message> for LogMessageIds {
    fn from(value: &serenity::Message) -> Self {
        Self {
            message: value.id,
            channel: value.channel_id,
            guild: value.guild_id,
            author: Some(value.author.id),
        }
    }
}

async fn is_excluded_message(
    ctx: &serenity::Context,
    guild_config: &GuildConfig,
    ids: LogMessageIds,
) -> bool {
    if ids.author == Some(ctx.cache.current_user().id) {
        return true;
    }

    if guild_config.logs_excluded_channels.contains(&ids.channel) {
        return true;
    }

    if let (Some(guild), Some(author)) = (ids.guild, ids.author) {
        if let Ok(member) = guild.member(&ctx.http, author).await {
            if member.roles(&ctx.cache).is_some_and(|roles| {
                roles
                    .iter()
                    .any(|role| role.has_permission(serenity::Permissions::ADMINISTRATOR))
            }) {
                return true;
            }
        }
    }

    false
}

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if let Some(guild_id) = message.guild_id {
        if let Some(storage) = &ctx.data::<crate::Data>().storage {
            let guild_config = storage.get_config(guild_id).await?;

            if is_excluded_message(ctx, &guild_config, message.into()).await {
                return Ok(());
            }

            storage.set_message_log(message.id, &message.into()).await?;
        }
    }

    Ok(())
}

fn make_link_components<'a>(
    link: impl Into<Cow<'a, str>>,
    label: impl Into<Cow<'a, str>>,
) -> Vec<serenity::CreateActionRow<'a>> {
    vec![serenity::CreateActionRow::Buttons(
        vec![serenity::CreateButton::new_link(link).label(label)].into(),
    )]
}

#[tracing::instrument(skip_all, fields(id = ids.message.get()))]
pub async fn edit(
    ctx: &serenity::Context,
    ids: LogMessageIds,
    prev_content: &str,
    new_content: &str,
    attachments: &[serenity::Attachment],
    timestamp: &chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    if let Some(guild_id) = ids.guild {
        if let Some(storage) = &ctx.data::<crate::Data>().storage {
            let guild_config = storage.get_config(guild_id).await?;

            if is_excluded_message(ctx, &guild_config, ids).await {
                return Ok(());
            }

            if prev_content == new_content {
                return Ok(());
            }

            if let Some(logs_channel) = guild_config.message_logs_channel {
                let mut embed_author = serenity::CreateEmbedAuthor::new("Message Edited");
                if let Some(author) = ids.author {
                    if let Ok(user) = author.to_user(&ctx).await {
                        embed_author = embed_author.icon_url(user.face());
                    }
                }

                let mut embed = serenity::CreateEmbed::default()
                    .author(embed_author)
                    .field(
                        "Channel",
                        utils::serenity::format_mentionable(Some(ids.channel)),
                        false,
                    )
                    .field(
                        "Author",
                        utils::serenity::format_mentionable(ids.author),
                        false,
                    )
                    .field(
                        "Previous content",
                        utils::truncate(prev_content, 1024),
                        false,
                    )
                    .field("New content", utils::truncate(new_content, 1024), false)
                    .color(0xffd43b)
                    .timestamp(serenity::Timestamp::from(*timestamp));

                if !attachments.is_empty() {
                    embed = embed.field(
                        "Attachments",
                        utils::serenity::format_attachments(attachments),
                        false,
                    );
                }

                logs_channel
                    .send_message(
                        &ctx.http,
                        serenity::CreateMessage::default()
                            .embed(embed)
                            .components(make_link_components(ids.link(), "Jump")),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = ids.message.get()))]
pub async fn delete(
    ctx: &serenity::Context,
    ids: LogMessageIds,
    log: &MessageLog,
    timestamp: &chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    if let Some(guild_id) = ids.guild {
        if let Some(storage) = &ctx.data::<crate::Data>().storage {
            let guild_config = storage.get_config(guild_id).await?;

            if is_excluded_message(ctx, &guild_config, ids).await {
                return Ok(());
            }

            if let Some(logs_channel) = guild_config.message_logs_channel {
                let mut embed_author = serenity::CreateEmbedAuthor::new("Message Deleted");
                if let Some(author) = ids.author {
                    if let Ok(user) = author.to_user(&ctx).await {
                        embed_author = embed_author.icon_url(user.face());
                    }
                }

                let mut embed = serenity::CreateEmbed::default()
                    .author(embed_author)
                    .field(
                        "Channel",
                        utils::serenity::format_mentionable(Some(ids.channel)),
                        false,
                    )
                    .field(
                        "Author",
                        utils::serenity::format_mentionable(ids.author),
                        false,
                    )
                    .field("Content", &log.content, false)
                    .color(0xff6b6b)
                    .timestamp(serenity::Timestamp::from(*timestamp));

                if !log.attachments.is_empty() {
                    embed = embed.field(
                        "Attachments",
                        utils::serenity::format_attachments(&log.attachments),
                        false,
                    );
                }

                logs_channel
                    .send_message(
                        &ctx.http,
                        serenity::CreateMessage::default()
                            .embed(embed)
                            .components(make_link_components(ids.link(), "Jump")),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = member.user.id.get()))]
pub async fn member_join(ctx: &serenity::Context, member: &serenity::Member) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        let guild_config = storage.get_config(member.guild_id).await?;

        if let Some(logs_channel) = guild_config.member_logs_channel {
            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default().embed(
                        serenity::CreateEmbed::default()
                            .author(
                                serenity::CreateEmbedAuthor::new(format!(
                                    "@{} joined",
                                    member.user.tag()
                                ))
                                .icon_url(member.user.face()),
                            )
                            .field("User", member.user.to_string(), false)
                            .field("ID", format!("`{}`", member.user.id), false)
                            .color(0x69db7c)
                            .timestamp(serenity::Timestamp::now()),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = user.id.get()))]
pub async fn member_leave(
    ctx: &serenity::Context,
    user: &serenity::User,
    member: Option<&serenity::Member>,
    guild_id: serenity::GuildId,
) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        let guild_config = storage.get_config(guild_id).await?;

        if let Some(logs_channel) = guild_config.member_logs_channel {
            let mut embed = serenity::CreateEmbed::default()
                .author(
                    serenity::CreateEmbedAuthor::new(format!("@{} left", user.tag()))
                        .icon_url(user.face()),
                )
                .field("User", user.to_string(), false)
                .field("ID", format!("`{}`", user.id), false)
                .color(0xff6b6b)
                .timestamp(serenity::Timestamp::now());

            if let Some(member) = member {
                if let Some(roles) = member.roles(&ctx.cache) {
                    embed = embed.field(
                        "Roles",
                        if roles.is_empty() {
                            "*None*".to_owned()
                        } else {
                            roles
                                .iter()
                                .map(|r| r.to_string())
                                .collect::<Vec<_>>()
                                .join(" ")
                        },
                        false,
                    );
                }

                if let Some(joined_at) = member.joined_at {
                    embed =
                        embed.field("Joined", format!("<t:{}:F>", joined_at.timestamp()), false);
                }
            }

            logs_channel
                .send_message(&ctx.http, serenity::CreateMessage::default().embed(embed))
                .await?;
        }
    }

    Ok(())
}
