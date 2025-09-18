// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::{
    CreateReply,
    serenity_prelude::{
        ChannelType, CreateActionRow, CreateComponent, CreateContainer, CreateSelectMenu,
        CreateSelectMenuKind, CreateTextDisplay, MessageFlags,
    },
};

use crate::Context;

// fn parse_id_set<T>(s: &str) -> Result<HashSet<T>>
// where
//     T: FromStr + Eq + Hash,
//     <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
// {
//     s.split([','])
//         .map(|f| f.trim().parse::<T>())
//         .collect::<Result<HashSet<_>, _>>()
//         .map_err(|err| err.into())
// }

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("edit", "starboard", "raw", "reset"),
    subcommand_required,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn config(ctx: Context<'_>) -> Result<()> {
    Ok(())
}

/// Edit the guild configuration interactively
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
#[expect(clippy::too_many_lines)]
async fn edit(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let data = storage.get_config(guild_id).await?;

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[
                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Starboard channel**\n-# Starboard channel to use for channels viewable by @everyone",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:starboard_channel",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                match data.starboard_channel {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Private category**\n-# Private category that uses a separate starboard",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:private_category",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Category].into()),
                            default_channels: Some(
                                match data.private_category {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Private starboard channel**\n-# Separate starboard channel to use for the private category",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:private_starboard_channel",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                match data.private_starboard_channel {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Moderation logs channel**\n-# Channel for moderation logs (e.g. bans, timeouts)",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:moderation_logs_channel",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                match data.moderation_logs_channel {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Message logs channel**\n-# Channel for message logs (e.g. edits, deletes)",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:message_logs_channel",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                match data.message_logs_channel {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Member logs channel**\n-# Channel for member logs (e.g. joins, leaves)",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:member_logs_channel",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                match data.member_logs_channel {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Logs excluded channels**\n-# List of channels excluded from message logs",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:logs_excluded_channels",
                        CreateSelectMenuKind::Channel {
                            channel_types: Some(vec![ChannelType::Text].into()),
                            default_channels: Some(
                                data.logs_excluded_channels.iter().copied().collect::<Vec<_>>().into()
                            ),
                        },
                    )
                    .min_values(0)
                    .max_values(25)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Moderator role**\n-# Role that moderators are assigned to, used for mentions",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:moderator_role",
                        CreateSelectMenuKind::Role {
                            default_roles: Some(
                                match data.moderator_role {
                                    Some(c) => vec![c],
                                    None => vec![],
                                }
                                .into(),
                            ),
                        },
                    )
                    .min_values(0)
                )),

                CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "**Random color roles**\n-# List of roles to rotate colors for daily",
                )),
                CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "cfg:random_color_roles",
                        CreateSelectMenuKind::Role {
                            default_roles: Some(
                                data.random_color_roles.iter().copied().collect::<Vec<_>>().into(),
                            ),
                        },
                    )
                    .min_values(0)
                    .max_values(10)
                )),
            ]),
    )
    .await?;

    Ok(())
}

/// Manage additional starboard configs
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn starboard(
    ctx: Context<'_>,

    #[description = "Comma separated list of starboard emojis; `*` matches all emojis"]
    starboard_emojis: Option<String>,
    #[description = "Threshold of reactions for messages to be shown on the starboard"]
    starboard_threshold: Option<u64>,
    #[description = "Clear both emoji and threshold configs"]
    #[flag]
    clear: bool,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let mut data = storage.get_config(guild_id).await?;

    if clear {
        data.starboard_emojis = None;
        data.starboard_threshold = None;
    } else {
        if let Some(emojis) = &starboard_emojis {
            data.starboard_emojis = Some(emojis.to_owned());
        }
        if let Some(threshold) = &starboard_threshold {
            data.starboard_threshold = Some(*threshold);
        }
    }

    storage.set_config(guild_id, &data).await?;

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(
                CreateContainer::new(&[
                    CreateComponent::TextDisplay(CreateTextDisplay::new("### Configuration")),
                    CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                        "```json\n{}\n```",
                        serde_json::to_string_pretty(&data)?
                    ))),
                ])
                .accent_color(0x63e6be),
            )]),
    )
    .await?;

    Ok(())
}

/// View the guild configuration in raw JSON
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn raw(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    let data = storage.get_config(guild_id).await?;

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(
                CreateContainer::new(&[
                    CreateComponent::TextDisplay(CreateTextDisplay::new("### Configuration")),
                    CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                        "```json\n{}\n```",
                        serde_json::to_string_pretty(&data)?
                    ))),
                ])
                .accent_color(0x63e6be),
            )]),
    )
    .await?;

    Ok(())
}

/// Reset the guild configuration to defaults
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
async fn reset(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| eyre!("could not obtain guild ID"))?;

    let data = ctx.data();
    let storage = data
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?;

    storage.del_config(guild_id).await?;

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(
                CreateContainer::new(&[CreateComponent::TextDisplay(CreateTextDisplay::new(
                    "### Reset configuration",
                ))])
                .accent_color(0x63e6be),
            )]),
    )
    .await?;

    Ok(())
}
