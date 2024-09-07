use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{self as serenity};

use crate::{utils::serenity::unique_username, Data};

pub async fn handle_message(message: &serenity::Message, data: &Data) -> Result<()> {
    if let Some(storage) = &data.storage {
        storage
            .set_message_log(&message.id.to_string(), &message.into())
            .await?;
    }

    Ok(())
}

static MESSAGE_LOGS_CHANNEL: Lazy<Option<serenity::ChannelId>> = Lazy::new(|| {
    std::env::var("MESSAGE_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<serenity::ChannelId>().ok())
});

static MEMBER_LOGS_CHANNEL: Lazy<Option<serenity::ChannelId>> = Lazy::new(|| {
    std::env::var("MEMBER_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<serenity::ChannelId>().ok())
});

fn make_link_components<'a>(link: &'a str, label: &'a str) -> Vec<serenity::CreateActionRow<'a>> {
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new_link(link).label(label),
    ])]
}

fn format_user(user: Option<&serenity::UserId>) -> String {
    user.map_or_else(
        || "*Unknown*".to_owned(),
        |user| format!("<@{user}> ({user})"),
    )
}

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
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    if author == &Some(ctx.cache.current_user().id) {
        return Ok(());
    }

    if let Some(logs_channel) = *MESSAGE_LOGS_CHANNEL {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Edited");
        if let Some(author) = author {
            embed_author = embed_author.icon_url(author.to_user(&ctx).await?.face());
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .author(embed_author)
                            .field("Channel", format!("<#{channel}>"), false)
                            .field(
                                "Previous content",
                                prev_content.as_ref().unwrap_or(&"*Unknown*".to_owned()),
                                false,
                            )
                            .field("New content", new_content, false)
                            .field("Author", format_user(author.as_ref()), false)
                            .color(0xffd43b)
                            .timestamp(timestamp),
                    )
                    .components(make_link_components(&link, "Jump")),
            )
            .await?;
    }

    Ok(())
}

pub async fn delete(
    ctx: &serenity::Context,
    (id, channel, guild): (
        &serenity::MessageId,
        &serenity::ChannelId,
        &Option<serenity::GuildId>,
    ),
    author: &Option<serenity::UserId>,
    content: &Option<String>,
    timestamp: &serenity::Timestamp,
) -> Result<()> {
    if author == &Some(ctx.cache.current_user().id) {
        return Ok(());
    }

    if let Some(logs_channel) = *MESSAGE_LOGS_CHANNEL {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Deleted");
        if let Some(author) = author {
            embed_author = embed_author.icon_url(author.to_user(&ctx).await?.face());
        }

        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .author(embed_author)
                            .field("Channel", format!("<#{channel}>"), false)
                            .field(
                                "Content",
                                content.as_ref().unwrap_or(&"*Unknown*".to_owned()),
                                false,
                            )
                            .field("Author", format_user(author.as_ref()), false)
                            .color(0xff6b6b)
                            .timestamp(timestamp),
                    )
                    .components(make_link_components(&link, "Jump")),
            )
            .await?;
    }

    Ok(())
}

pub async fn member_join(ctx: &serenity::Context, user: &serenity::User) -> Result<()> {
    if let Some(logs_channel) = *MEMBER_LOGS_CHANNEL {
        logs_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default().embed(
                    serenity::CreateEmbed::default()
                        .author(
                            serenity::CreateEmbedAuthor::new(format!(
                                "@{} joined",
                                unique_username(user)
                            ))
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

pub async fn member_leave(
    ctx: &serenity::Context,
    user: &serenity::User,
    member: &Option<serenity::Member>,
) -> Result<()> {
    if let Some(logs_channel) = *MEMBER_LOGS_CHANNEL {
        let mut embed = serenity::CreateEmbed::default()
            .author(
                serenity::CreateEmbedAuthor::new(format!("@{} left", unique_username(user)))
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
