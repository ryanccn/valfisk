// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable as _};

use crate::config::CONFIG;

async fn get_starboard_channel(
    http: impl serenity::CacheHttp,
    channel: serenity::ChannelId,
    guild: Option<serenity::GuildId>,
) -> Result<Option<serenity::ChannelId>> {
    let Some(guild_channel) = channel.to_guild_channel(&http, guild).await.ok() else {
        return Ok(None);
    };

    if CONFIG.private_category.is_some() && CONFIG.private_category == guild_channel.parent_id {
        return Ok(CONFIG.private_starboard_channel);
    }

    Ok(CONFIG.starboard_channel)
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

impl std::str::FromStr for StarboardEmojis {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

fn is_significant_reaction(reaction: &serenity::MessageReaction) -> bool {
    CONFIG.starboard_emojis.allow(reaction) && reaction.count >= CONFIG.starboard_threshold
}

fn get_significant_reactions(message: &serenity::Message) -> Vec<(serenity::ReactionType, u64)> {
    let mut collected_reactions: Vec<(serenity::ReactionType, u64)> = message
        .reactions
        .iter()
        .filter(|r| is_significant_reaction(r))
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

    if let Some(guild_id) = message.guild_channel(&ctx).await.ok().map(|ch| ch.guild_id) {
        if let Ok(member) = guild_id.member(&ctx, message.author.id).await {
            for role_id in &member.roles {
                if let Ok(role) = guild_id.role(&ctx.http, *role_id).await {
                    if role.colour.0 != 0x99aab5 {
                        builder = builder.color(role.colour);
                        break;
                    }
                }
            }
        }
    }

    builder
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        if let Some(starboard) =
            get_starboard_channel(&ctx, message.channel_id, message.guild_id).await?
        {
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

                    tracing::debug!(
                        "Deleted starboard message {} for {}",
                        existing_starboard_message,
                        message.id
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
                        "Edited starboard message {} for {}",
                        existing_starboard_message,
                        message.id
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

                tracing::debug!(
                    "Created starboard message {} for {}",
                    starboard_message.id,
                    message.id
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
    channel_id: serenity::ChannelId,
    guild_id: Option<serenity::GuildId>,
) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        if let Some(starboard_channel) = get_starboard_channel(&ctx, channel_id, guild_id).await? {
            if let Some(starboard_id) = storage.get_starboard(deleted_message_id.get()).await? {
                tracing::debug!(
                    "Deleted starboard message {} for {} (source deleted)",
                    starboard_id,
                    deleted_message_id
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
