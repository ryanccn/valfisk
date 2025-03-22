// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use humansize::{FormatSizeOptions, format_size};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use eyre::Result;

use crate::{config::CONFIG, storage::log::MessageLog, utils};

#[derive(Debug, Clone, Copy)]
pub struct LogMessageIds {
    pub message: serenity::MessageId,
    pub channel: serenity::ChannelId,
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

async fn is_excluded_message(ctx: &serenity::Context, ids: LogMessageIds) -> bool {
    if ids.author == Some(ctx.cache.current_user().id) {
        return true;
    }

    if CONFIG.logs_excluded_channels.contains(&ids.channel) {
        return true;
    }

    if let Some(guild) = ids.guild {
        if let Some(author) = ids.author {
            if let Ok(author_member) = guild.member(&ctx.http, author).await {
                if utils::serenity::is_administrator(ctx, &author_member) {
                    return true;
                }
            }
        }
    }

    false
}

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if is_excluded_message(ctx, message.into()).await {
        return Ok(());
    }

    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        storage
            .set_message_log(message.id.get(), &message.into())
            .await?;
    }

    Ok(())
}

fn make_link_components<'a>(link: &'a str, label: &'a str) -> Vec<serenity::CreateActionRow<'a>> {
    vec![serenity::CreateActionRow::Buttons(
        vec![serenity::CreateButton::new_link(link).label(label)].into(),
    )]
}

pub fn format_user(user: Option<&serenity::UserId>) -> String {
    user.map_or_else(
        || "*Unknown*".to_owned(),
        |user| format!("{} ({user})", user.mention()),
    )
}

fn format_attachments(attachments: &[serenity::Attachment]) -> String {
    attachments
        .iter()
        .map(|att| {
            format!(
                "[{}]({}) ({})",
                att.filename,
                att.url,
                format_size(
                    att.size,
                    FormatSizeOptions::default().space_after_value(true)
                )
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tracing::instrument(skip_all, fields(id = ids.message.get()))]
pub async fn edit(
    ctx: &serenity::Context,
    ids: LogMessageIds,
    prev_content: Option<&str>,
    new_content: &str,
    attachments: &[serenity::Attachment],
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    if is_excluded_message(ctx, ids).await {
        return Ok(());
    }

    if let Some(logs_channel) = CONFIG.message_logs_channel {
        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Edited");
        if let Some(author) = ids.author {
            if let Ok(user) = author.to_user(&ctx).await {
                embed_author = embed_author.icon_url(user.face());
            }
        }

        let mut embed = serenity::CreateEmbed::default()
            .author(embed_author)
            .field("Channel", ids.channel.mention().to_string(), false)
            .field(
                "Previous content",
                prev_content.map_or_else(|| "*Unknown*".to_owned(), |s| utils::truncate(s, 1024)),
                false,
            )
            .field("New content", utils::truncate(new_content, 1024), false)
            .field("Author", format_user(ids.author.as_ref()), false)
            .color(0xffd43b)
            .timestamp(timestamp);

        if !attachments.is_empty() {
            embed = embed.field("Attachments", format_attachments(attachments), false);
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(embed)
                    .components(make_link_components(&ids.link(), "Jump")),
            )
            .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = ids.message.get()))]
pub async fn delete(
    ctx: &serenity::Context,
    ids: LogMessageIds,
    log: &MessageLog,
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    if is_excluded_message(ctx, ids).await {
        return Ok(());
    }

    if let Some(logs_channel) = CONFIG.message_logs_channel {
        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Deleted");
        if let Some(author) = ids.author {
            if let Ok(user) = author.to_user(&ctx).await {
                embed_author = embed_author.icon_url(user.face());
            }
        }

        let mut embed = serenity::CreateEmbed::default()
            .author(embed_author)
            .field("Channel", ids.channel.mention().to_string(), false)
            .field("Content", &log.content, false)
            .field("Author", format_user(ids.author.as_ref()), false)
            .color(0xff6b6b)
            .timestamp(timestamp);

        if !log.attachments.is_empty() {
            embed = embed.field("Attachments", format_attachments(&log.attachments), false);
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(embed)
                    .components(make_link_components(&ids.link(), "Jump")),
            )
            .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = user.id.get()))]
pub async fn member_join(ctx: &serenity::Context, user: &serenity::User) -> Result<()> {
    if let Some(logs_channel) = CONFIG.member_logs_channel {
        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default().embed(
                    serenity::CreateEmbed::default()
                        .author(
                            serenity::CreateEmbedAuthor::new(format!("@{} joined", user.tag()))
                                .icon_url(user.face()),
                        )
                        .field("User", user.to_string(), false)
                        .field("ID", format!("`{}`", user.id), false)
                        .color(0x69db7c)
                        .timestamp(serenity::Timestamp::now()),
                ),
            )
            .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = user.id.get()))]
pub async fn member_leave(
    ctx: &serenity::Context,
    user: &serenity::User,
    member: Option<&serenity::Member>,
) -> Result<()> {
    if let Some(logs_channel) = CONFIG.member_logs_channel {
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
                embed = embed.field("Joined", format!("<t:{}:F>", joined_at.timestamp()), false);
            }
        }

        logs_channel
            .send_message(&ctx.http, serenity::CreateMessage::default().embed(embed))
            .await?;
    }

    Ok(())
}
