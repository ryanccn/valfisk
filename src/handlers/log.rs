use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;

use crate::Data;

pub async fn handle_message(message: &serenity::Message, data: &Data) -> Result<()> {
    if let Some(storage) = &data.storage {
        storage
            .set_message_log(&message.id.to_string(), &message.into())
            .await?;
    }

    Ok(())
}

static LOGS_CHANNEL: Lazy<Option<serenity::ChannelId>> = Lazy::new(|| {
    std::env::var("MESSAGE_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<serenity::ChannelId>().ok())
});

fn make_link_components(link: &str, label: &str) -> Vec<serenity::CreateActionRow> {
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new_link(link).label(label),
    ])]
}

fn format_user(user: Option<&serenity::UserId>) -> String {
    match user {
        Some(user) => format!("<@{user}> ({user})"),
        None => "*Unknown*".to_owned(),
    }
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
    if let Some(logs_channel) = *LOGS_CHANNEL {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Edited");
        if let Some(author) = author {
            embed_author = embed_author.icon_url(author.to_user(&ctx).await?.face());
        }

        logs_channel
            .send_message(
                &ctx,
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
                            .color(0xfde047)
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
    if let Some(logs_channel) = *LOGS_CHANNEL {
        let link = id.link(channel.to_owned(), guild.to_owned());

        let mut embed_author = serenity::CreateEmbedAuthor::new("Message Deleted");
        if let Some(author) = author {
            embed_author = embed_author.icon_url(author.to_user(&ctx).await?.face());
        }

        logs_channel
            .send_message(
                &ctx,
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
                            .color(0xef4444)
                            .timestamp(timestamp),
                    )
                    .components(make_link_components(&link, "Jump")),
            )
            .await?;
    }

    Ok(())
}
