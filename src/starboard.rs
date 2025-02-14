// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env;

use poise::serenity_prelude::{self as serenity, Mentionable as _};

use eyre::Result;
use tracing::debug;

use crate::config::CONFIG;

async fn get_starboard_channel(
    http: impl serenity::CacheHttp,
    message_channel: &serenity::ChannelId,
) -> Result<Option<serenity::ChannelId>> {
    let Some(message_channel) = message_channel.to_channel(&http, None).await?.guild() else {
        return Ok(None);
    };

    if CONFIG.fren_category == message_channel.parent_id && CONFIG.fren_starboard_channel.is_some()
    {
        return Ok(CONFIG.fren_starboard_channel);
    }

    Ok(CONFIG.starboard_channel)
}

#[allow(clippy::redundant_closure_for_method_calls)]
fn is_significant_reaction(reaction: &serenity::MessageReaction) -> Result<bool> {
    let threshold = env::var("STARBOARD_THRESHOLD").map_or(Ok(3), |s| s.parse::<u64>())?;

    if reaction.count < threshold {
        return Ok(false);
    }

    Ok(match env::var("STARBOARD_EMOJIS") {
        Ok(starboard_emojis) if starboard_emojis == "ANY" => true,

        Ok(starboard_emojis) => {
            let emojis = starboard_emojis
                .split(',')
                .map(|c| c.trim().to_owned())
                .collect::<Vec<_>>();

            match &reaction.reaction_type {
                serenity::ReactionType::Custom { id, .. } => emojis.contains(&id.to_string()),
                serenity::ReactionType::Unicode(fixed_string) => {
                    emojis.contains(&fixed_string.to_string())
                }
                _ => false,
            }
        }

        Err(_) => false,
    })
}

fn get_significant_reactions(message: &serenity::Message) -> Vec<(serenity::ReactionType, u64)> {
    let mut collected_reactions: Vec<(serenity::ReactionType, u64)> = message
        .reactions
        .iter()
        .filter(|r| is_significant_reaction(r).is_ok_and(|b| b))
        .map(|r| (r.reaction_type.clone(), r.count))
        .collect();

    collected_reactions.sort_by_key(|i| match &i.0 {
        serenity::ReactionType::Custom { id, .. } => id.get().to_string(),
        serenity::ReactionType::Unicode(str) => str.to_string(),
        _ => "unknown".to_owned(),
    });

    collected_reactions
}

fn serialize_reactions(
    channel: serenity::ChannelId,
    reactions: &[(serenity::ReactionType, u64)],
) -> String {
    let reaction_string = reactions
        .iter()
        .map(|r| format!("{} {}", r.0, r.1))
        .collect::<Vec<_>>()
        .join(" ");

    format!("**{reaction_string}** in {}", channel.mention())
}

async fn make_message_embed<'a>(
    ctx: &serenity::Context,
    message: &serenity::Message,
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

    if let Some(reference) = &message.message_reference {
        if let Some(message_id) = reference.message_id {
            builder = builder.field(
                "Replying to",
                message_id.link(reference.channel_id, reference.guild_id),
                false,
            );
        }
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
        .channel_id
        .to_guild_channel(&ctx, None)
        .await
        .ok()
        .map(|c| c.guild_id)
    {
        if let Ok(member) = guild_id.member(&ctx, message.author.id).await {
            if let Some(top_role) = member.roles(&ctx.cache).unwrap_or_default().first() {
                builder = builder.color(top_role.colour);
            }
        }
    }

    builder
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(
    ctx: &serenity::Context,
    data: &crate::Data,
    message: &serenity::Message,
) -> Result<()> {
    if let Some(storage) = &data.storage {
        if let Some(starboard) = get_starboard_channel(&ctx, &message.channel_id).await? {
            let significant_reactions = get_significant_reactions(message);

            if let Some(existing_starboard_message) = storage
                .get_starboard(message.id.get())
                .await?
                .map(|s| s.into())
            {
                if significant_reactions.is_empty() {
                    starboard
                        .delete_message(&ctx.http, existing_starboard_message, None)
                        .await?;

                    storage.del_starboard(message.id.get()).await?;

                    debug!(
                        "Deleted starboard message {} for {}",
                        existing_starboard_message, message.id
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

                    debug!(
                        "Edited starboard message {} for {}",
                        existing_starboard_message, message.id
                    );
                }
            } else if !significant_reactions.is_empty() {
                let content = serialize_reactions(message.channel_id, &significant_reactions);
                let embed = make_message_embed(ctx, message).await;

                let row = serenity::CreateActionRow::Buttons(
                    vec![serenity::CreateButton::new_link(message.link()).label("Jump to message")]
                        .into(),
                );

                let starboard_message = starboard
                    .send_message(
                        &ctx.http,
                        serenity::CreateMessage::default()
                            .content(content)
                            .embed(embed)
                            .components(vec![row]),
                    )
                    .await?;

                storage
                    .set_starboard(message.id.get(), &starboard_message.id.get())
                    .await?;

                debug!(
                    "Created starboard message {} for {}",
                    starboard_message.id, message.id
                );
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip(ctx, data))]
pub async fn handle_deletion(
    ctx: &serenity::Context,
    data: &crate::Data,
    deleted_message_id: &serenity::MessageId,
    channel_id: &serenity::ChannelId,
) -> Result<()> {
    if let Some(storage) = &data.storage {
        if let Some(starboard_channel) = get_starboard_channel(&ctx, channel_id).await? {
            if let Some(starboard_id) = storage.get_starboard(deleted_message_id.get()).await? {
                debug!(
                    "Deleted starboard message {} for {} (source deleted)",
                    starboard_id, deleted_message_id
                );

                storage.del_starboard(deleted_message_id.get()).await?;

                ctx.http
                    .delete_message(starboard_channel, starboard_id.into(), None)
                    .await?;
            }
        }
    }

    Ok(())
}
