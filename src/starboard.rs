use std::env;

use poise::serenity_prelude as serenity;

use color_eyre::eyre::{OptionExt, Result};
use log::debug;

fn channel_from_env(key: &str) -> Option<serenity::ChannelId> {
    env::var(key)
        .ok()
        .and_then(|s| s.parse::<serenity::ChannelId>().ok())
}

async fn get_starboard_channel(
    http: impl serenity::CacheHttp,
    message_channel: serenity::ChannelId,
) -> Result<Option<serenity::ChannelId>> {
    let Some(message_channel) = message_channel.to_channel(&http).await?.guild() else {
        return Ok(None);
    };

    if channel_from_env("FREN_CATEGORY") == message_channel.parent_id {
        if let Some(fren_category) = channel_from_env("FREN_STARBOARD_CHANNEL") {
            return Ok(Some(fren_category));
        }
    }

    if let Some(fren_category) = channel_from_env("STARBOARD_CHANNEL") {
        return Ok(Some(fren_category));
    }

    Ok(None)
}

#[allow(clippy::redundant_closure_for_method_calls)]
fn is_significant_reaction(reaction: &serenity::MessageReaction) -> Result<bool> {
    let threshold = env::var("STARBOARD_THRESHOLD").map_or(Ok(3), |s| s.parse::<u64>())?;

    if reaction.count < threshold {
        return Ok(false);
    }

    Ok(
        match env::var("STARBOARD_EMOJIS").map(|v| {
            v.split(',')
                .map(|c| c.trim().to_owned())
                .collect::<Vec<_>>()
        }) {
            Ok(emojis) => match &reaction.reaction_type {
                serenity::ReactionType::Custom { id, .. } => emojis.contains(&id.to_string()),
                serenity::ReactionType::Unicode(fixed_string) => {
                    emojis.contains(&fixed_string.to_string())
                }
                _ => false,
            },

            Err(_) => false,
        },
    )
}

fn get_significant_reactions(
    message: &serenity::Message,
) -> Result<Vec<(serenity::ReactionType, u64)>> {
    let mut collected_reactions: Vec<(serenity::ReactionType, u64)> = Vec::new();

    for reaction in &message.reactions {
        if is_significant_reaction(reaction)? {
            collected_reactions.push((reaction.reaction_type.clone(), reaction.count));
        };
    }

    collected_reactions.sort_by_key(|i| match &i.0 {
        serenity::ReactionType::Custom { id, .. } => id.get().to_string(),
        serenity::ReactionType::Unicode(str) => str.to_string(),
        _ => "unknown".to_owned(),
    });

    Ok(collected_reactions)
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

    format!("**{reaction_string}** in <#{channel}>")
}

async fn make_message_embed<'a>(
    _http: &serenity::Http,
    message: &serenity::Message,
) -> Result<serenity::CreateEmbed<'a>> {
    let content = message.content.to_string();
    let mut builder = serenity::CreateEmbed::default()
        .description(if content.is_empty() {
            "*No content*".to_owned()
        } else {
            content
        })
        .author(
            serenity::CreateEmbedAuthor::new(message.author.name.to_string())
                .icon_url(message.author.face()),
        )
        .timestamp(message.timestamp);

    if let Some(reference) = message.message_reference.as_ref() {
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

    // TODO: implement top-most role color
    builder = builder.color(0xfcd34d);

    Ok(builder)
}

pub async fn handle(
    ctx: &serenity::Context,
    data: &crate::Data,
    message: &serenity::Message,
) -> Result<()> {
    let storage = data
        .storage
        .as_ref()
        .ok_or_eyre("no storage available for starboard features")?;

    if let Some(starboard) = get_starboard_channel(&ctx, message.channel_id).await? {
        let significant_reactions = get_significant_reactions(message)?;

        if let Some(existing_starboard_message) = storage
            .get_starboard(&message.id.to_string())
            .await?
            .and_then(|s| s.parse::<serenity::MessageId>().ok())
        {
            if significant_reactions.is_empty() {
                starboard
                    .delete_message(&ctx, existing_starboard_message)
                    .await?;

                storage.del_starboard(&message.id.to_string()).await?;

                debug!(
                    "Deleted starboard message {} for {}",
                    existing_starboard_message, message.id
                );
            } else {
                starboard
                    .edit_message(
                        &ctx,
                        &existing_starboard_message,
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
            let embed = make_message_embed(&ctx.http, message).await?;

            let link_button =
                serenity::CreateButton::new_link(message.link()).label("Jump to message");
            let row = serenity::CreateActionRow::Buttons(vec![link_button]);

            let starboard_message = starboard
                .send_message(
                    &ctx,
                    serenity::CreateMessage::default()
                        .content(content)
                        .embed(embed)
                        .components(vec![row]),
                )
                .await?;

            storage
                .set_starboard(&message.id.to_string(), &starboard_message.id.to_string())
                .await?;

            debug!(
                "Created starboard message {} for {}",
                starboard_message.id, message.id
            );
        }
    }

    Ok(())
}
