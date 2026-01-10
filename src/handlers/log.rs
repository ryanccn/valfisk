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
        self.message.link(self.channel, self.guild).to_string()
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

    if let (Some(guild), Some(author)) = (ids.guild, ids.author)
        && let Ok(member) = guild.member(&ctx, author).await
        && member.roles(&ctx.cache).is_some_and(|roles| {
            roles
                .iter()
                .any(|role| role.has_permission(serenity::Permissions::ADMINISTRATOR))
        })
    {
        return true;
    }

    if let (Some(guild), Some(author)) = (ids.guild, ids.author)
        && let Ok(guild) = guild.to_partial_guild(&ctx).await
        && guild.owner_id == author
    {
        return true;
    }

    false
}

#[tracing::instrument(skip_all, fields(id = message.id.get()))]
pub async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if let Some(guild_id) = message.guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        let guild_config = storage.get_config(guild_id).await?;

        if is_excluded_message(ctx, &guild_config, message.into()).await {
            return Ok(());
        }

        storage.set_message_log(message.id, &message.into()).await?;
    }

    Ok(())
}

fn make_link_component<'a>(link: impl Into<Cow<'a, str>>) -> serenity::CreateComponent<'a> {
    serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![serenity::CreateButton::new_link(link).label("Message")].into(),
    ))
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
    if let Some(guild_id) = ids.guild
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        let guild_config = storage.get_config(guild_id).await?;

        if is_excluded_message(ctx, &guild_config, ids).await {
            return Ok(());
        }

        if prev_content == new_content {
            return Ok(());
        }

        if let Some(logs_channel) = guild_config.message_logs_channel {
            let mut container = serenity::CreateContainer::new(vec![
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    "### Message Edited",
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Channel**\n{}",
                        utils::serenity::format_mentionable(Some(ids.channel))
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Author**\n{}",
                        utils::serenity::format_mentionable(ids.author)
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Previous content**\n{}",
                        utils::truncate(prev_content, 1024)
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!("**New content**\n{}", utils::truncate(new_content, 1024)),
                )),
            ])
            .accent_color(0xffd43b);

            if !attachments.is_empty() {
                container =
                    container.add_component(serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Attachments**\n{}",
                            utils::serenity::format_attachments(attachments)
                        )),
                    ));
            }

            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "-# {}",
                    serenity::FormattedTimestamp::new((*timestamp).into(), None),
                )),
            ));

            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(&[
                            serenity::CreateComponent::Container(container),
                            make_link_component(ids.link()),
                        ]),
                )
                .await?;
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
    if let Some(guild_id) = ids.guild
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        let guild_config = storage.get_config(guild_id).await?;

        if is_excluded_message(ctx, &guild_config, ids).await {
            return Ok(());
        }

        if let Some(logs_channel) = guild_config.message_logs_channel {
            let mut container = serenity::CreateContainer::new(vec![
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    "### Message Deleted",
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Channel**\n{}",
                        utils::serenity::format_mentionable(Some(ids.channel))
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "**Author**\n{}",
                        utils::serenity::format_mentionable(ids.author)
                    ),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!("**Content**\n{}", utils::truncate(&log.content, 1024)),
                )),
            ])
            .accent_color(0xff6b6b);

            if !log.attachments.is_empty() {
                container =
                    container.add_component(serenity::CreateContainerComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "**Attachments**\n{}",
                            utils::serenity::format_attachments(&log.attachments)
                        )),
                    ));
            }

            container = container.add_component(serenity::CreateContainerComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!(
                    "-# {}",
                    serenity::FormattedTimestamp::new((*timestamp).into(), None),
                )),
            ));

            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(&[
                            serenity::CreateComponent::Container(container),
                            make_link_component(ids.link()),
                        ]),
                )
                .await?;
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
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(&[serenity::CreateComponent::Container(
                            serenity::CreateContainer::new(vec![
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new(format!(
                                        "### Member joined\n{}\n-# {}",
                                        utils::serenity::format_mentionable(Some(member.user.id)),
                                        serenity::FormattedTimestamp::now()
                                    )),
                                ),
                            ])
                            .accent_color(0x69db7c),
                        )]),
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
            let mut container = serenity::CreateContainer::new(vec![
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!(
                        "### Member left\n{}\n-# {}",
                        utils::serenity::format_mentionable(Some(user.id)),
                        serenity::FormattedTimestamp::now()
                    ),
                )),
            ])
            .accent_color(0xff6b6b);

            if let Some(member) = member {
                if let Some(roles) = member.roles(&ctx.cache) {
                    container =
                        container.add_component(serenity::CreateContainerComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(format!(
                                "**Roles**\n{}",
                                if roles.is_empty() {
                                    "*None*".to_owned()
                                } else {
                                    roles
                                        .iter()
                                        .map(|r| r.to_string())
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                },
                            )),
                        ));
                }

                if let Some(joined_at) = member.joined_at {
                    container =
                        container.add_component(serenity::CreateContainerComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(format!(
                                "**Joined at**\n{}",
                                serenity::FormattedTimestamp::new(joined_at, None)
                            )),
                        ));
                }
            }

            logs_channel
                .send_message(
                    &ctx.http,
                    serenity::CreateMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(serenity::CreateAllowedMentions::new())
                        .components(&[serenity::CreateComponent::Container(container)]),
                )
                .await?;
        }
    }

    Ok(())
}
