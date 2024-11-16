// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use humansize::{format_size, FormatSizeOptions};
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use eyre::Result;

use crate::{config::CONFIG, storage::log::MessageLog, utils, Data};

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn handle_message(
    ctx: &serenity::Context,
    data: &Data,
    message: &serenity::Message,
) -> Result<()> {
    if let Some(storage) = &data.storage {
        if message.author.id == ctx.cache.current_user().id {
            return Ok(());
        }

        storage
            .set_message_log(&message.id.to_string(), &message.into())
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

#[tracing::instrument(skip_all, fields(id = id.get()))]
pub async fn edit(
    ctx: &serenity::Context,
    (id, channel, guild): (
        &serenity::MessageId,
        &serenity::ChannelId,
        &Option<serenity::GuildId>,
    ),
    author: &Option<serenity::UserId>,
    prev_content: &Option<String>,
    new_content: &str,
    attachments: &[serenity::Attachment],
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    if author == &Some(ctx.cache.current_user().id) {
        return Ok(());
    }

    if let Some(logs_channel) = CONFIG.message_logs_channel {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Edited");
        if let Some(author) = author {
            if let Ok(user) = author.to_user(&ctx).await {
                embed_author = embed_author.icon_url(user.face());
            }
        }

        let mut embed = serenity::CreateEmbed::default()
            .author(embed_author)
            .field("Channel", channel.mention().to_string(), false)
            .field(
                "Previous content",
                prev_content
                    .clone()
                    .map_or_else(|| "*Unknown*".to_owned(), |s| utils::truncate(&s, 1024)),
                false,
            )
            .field("New content", utils::truncate(new_content, 1024), false)
            .field("Author", format_user(author.as_ref()), false)
            .color(0xffd43b)
            .timestamp(timestamp);

        if !attachments.is_empty() {
            embed = embed.field(
                "Attachments",
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
                    .join("\n"),
                false,
            );
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(embed)
                    .components(make_link_components(&link, "Jump")),
            )
            .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(id = id.get()))]
pub async fn delete(
    ctx: &serenity::Context,
    (id, channel, guild): (
        &serenity::MessageId,
        &serenity::ChannelId,
        &Option<serenity::GuildId>,
    ),
    log: &Option<MessageLog>,
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    let content = log.as_ref().and_then(|l| l.content.clone());
    let author = log.as_ref().and_then(|l| l.author);
    let attachments = log
        .as_ref()
        .map(|l| l.attachments.clone())
        .unwrap_or_default();

    if author == Some(ctx.cache.current_user().id) {
        return Ok(());
    }

    if let Some(logs_channel) = CONFIG.message_logs_channel {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Deleted");
        if let Some(author) = author {
            if let Ok(user) = author.to_user(&ctx).await {
                embed_author = embed_author.icon_url(user.face());
            }
        }

        let mut embed = serenity::CreateEmbed::default()
            .author(embed_author)
            .field("Channel", channel.mention().to_string(), false)
            .field(
                "Content",
                content.map_or_else(|| "*Unknown*".to_owned(), |s| utils::truncate(&s, 1024)),
                false,
            )
            .field("Author", format_user(author.as_ref()), false)
            .color(0xff6b6b)
            .timestamp(timestamp);

        if !attachments.is_empty() {
            embed = embed.field(
                "Attachments",
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
                    .join("\n"),
                false,
            );
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(embed)
                    .components(make_link_components(&link, "Jump")),
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
    member: &Option<serenity::Member>,
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
