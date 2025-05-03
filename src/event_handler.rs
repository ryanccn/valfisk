// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use crate::{commands, config::CONFIG, handlers, storage::log::MessageLog};

fn validate_commands(commands: &[poise::Command<crate::Data, eyre::Report>]) {
    if !commands.iter().filter(|c| c.guild_only).all(|c| {
        c.install_context
            .as_ref()
            .is_some_and(|i| i == &[serenity::InstallationContext::Guild])
    }) {
        panic!(
            "some commands marked as `guild_only` do not have installation contexts restricted to `Guild`"
        );
    }

    if !commands.iter().filter(|c| c.guild_only).all(|c| {
        c.interaction_context
            .as_ref()
            .is_some_and(|i| i == &[serenity::InteractionContext::Guild])
    }) {
        panic!(
            "some commands marked as `guild_only` do not have interaction contexts restricted to `Guild`"
        );
    }

    if !commands.iter().filter(|c| !c.guild_only).all(|c| {
        c.install_context.as_ref().is_some_and(|i| {
            i == &[
                serenity::InstallationContext::Guild,
                serenity::InstallationContext::User,
            ]
        })
    }) {
        panic!(
            "some commands not marked as `guild_only` do not have unrestricted installation contexts"
        );
    }

    if !commands
        .iter()
        .filter(|c| !c.guild_only && !c.owners_only)
        .all(|c| {
            c.interaction_context.as_ref().is_some_and(|i| {
                i == &[
                    serenity::InteractionContext::Guild,
                    serenity::InteractionContext::BotDm,
                    serenity::InteractionContext::PrivateChannel,
                ]
            })
        })
    {
        panic!(
            "some commands not marked as `guild_only` do not have unrestricted interaction contexts"
        );
    }

    if !commands
        .iter()
        .filter(|c| c.owners_only)
        .all(|c| c.default_member_permissions == serenity::Permissions::ADMINISTRATOR)
    {
        panic!(
            "some commands marked as `owners_only` do not have `default_member_permissions` set to ADMINISTRATOR"
        );
    }

    if !commands.iter().filter(|c| c.owners_only).all(|c| {
        c.install_context
            .as_ref()
            .is_some_and(|i| i == &[serenity::InstallationContext::Guild])
    }) {
        panic!(
            "some commands marked as `owners_only` do not have install contexts restricted to `Guild`"
        );
    }

    if !commands.iter().filter(|c| c.owners_only).all(|c| {
        c.interaction_context
            .as_ref()
            .is_some_and(|i| i == &[serenity::InteractionContext::Guild])
    }) {
        panic!(
            "some commands marked as `owners_only` do not have interaction contexts restricted to `Guild`"
        );
    }
}

pub struct EventHandler;

#[serenity::async_trait]
impl serenity::EventHandler for EventHandler {
    #[tracing::instrument(skip_all)]
    async fn dispatch(&self, ctx: &serenity::Context, event: &serenity::FullEvent) {
        use serenity::FullEvent;

        let outcome: eyre::Result<()> = async {
            match event {
                FullEvent::Ready { data_about_bot, .. } => {
                    tracing::info!(
                        user = data_about_bot.user.tag(),
                        app = data_about_bot.application.id.get(),
                        "connected to Discord"
                    );

                    let commands_data = commands::all();
                    validate_commands(&commands_data);

                    let (count, global, guild, owners) = (
                        commands_data.len(),
                        commands_data
                            .iter()
                            .filter(|c| !c.owners_only && !c.guild_only)
                            .map(|c| c.name.to_string())
                            .collect::<Vec<_>>(),
                        commands_data
                            .iter()
                            .filter(|c| !c.owners_only && c.guild_only)
                            .map(|c| c.name.to_string())
                            .collect::<Vec<_>>(),
                        commands_data
                            .iter()
                            .filter(|c| c.owners_only)
                            .map(|c| c.name.to_string())
                            .collect::<Vec<_>>(),
                    );

                    let (public_commands, owner_commands) = commands_data
                        .into_iter()
                        .partition::<Vec<_>, _>(|c| !c.owners_only);

                    poise::builtins::register_globally(&ctx.http, &public_commands).await?;

                    if let Some(guild) = CONFIG.admin_guild_id {
                        poise::builtins::register_in_guild(&ctx.http, &owner_commands, guild)
                            .await?;
                    }

                    tracing::info!(
                        count,
                        ?global,
                        ?guild,
                        ?owners,
                        "registered application commands"
                    );

                    commands::restore::presence(ctx).await?;
                    commands::restore::reminders(ctx).await?;
                }

                FullEvent::Message { new_message, .. } => {
                    if new_message.guild_id.is_some() {
                        Box::pin(handlers::message_guild(ctx, new_message)).await?;
                    } else {
                        handlers::message_dm(ctx, new_message).await?;
                    }
                }

                FullEvent::MessageUpdate { event, .. } => {
                    if event.message.guild_id.is_none() {
                        return Ok(());
                    }

                    let timestamp = event
                        .message
                        .edited_timestamp
                        .unwrap_or_else(serenity::Timestamp::now);

                    if let Some(storage) = &ctx.data::<crate::Data>().storage {
                        let prev_data = storage.get_message_log(event.message.id).await?;

                        let new_content = event.message.content.as_str();
                        let attachments = event.message.attachments.to_vec();

                        storage
                            .set_message_log(
                                event.message.id,
                                &MessageLog::new(
                                    new_content,
                                    event.message.author.id,
                                    attachments.clone(),
                                ),
                            )
                            .await?;

                        if let Some(prev_content) = &prev_data.map(|p| p.content) {
                            handlers::log::edit(
                                ctx,
                                handlers::log::LogMessageIds {
                                    message: event.message.id,
                                    channel: event.message.channel_id,
                                    guild: event.message.guild_id,
                                    author: Some(event.message.author.id),
                                },
                                prev_content,
                                new_content,
                                &attachments,
                                &timestamp,
                            )
                            .await?;
                        }
                    }
                }

                FullEvent::MessageDelete {
                    channel_id,
                    deleted_message_id,
                    guild_id,
                    ..
                } => {
                    if guild_id.is_none() {
                        return Ok(());
                    }

                    handlers::starboard::handle_deletion(
                        ctx,
                        *deleted_message_id,
                        *channel_id,
                        *guild_id,
                    )
                    .await?;

                    let timestamp = serenity::Timestamp::now();

                    if let Some(storage) = &ctx.data::<crate::Data>().storage {
                        if let Some(logged_data) =
                            storage.get_message_log(*deleted_message_id).await?
                        {
                            handlers::log::delete(
                                ctx,
                                handlers::log::LogMessageIds {
                                    message: *deleted_message_id,
                                    channel: *channel_id,
                                    guild: *guild_id,
                                    author: Some(logged_data.author),
                                },
                                &logged_data,
                                &timestamp,
                            )
                            .await?;

                            storage.del_message_log(*deleted_message_id).await?;
                        }
                    }
                }

                FullEvent::ReactionAdd { add_reaction, .. } => {
                    if add_reaction.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = add_reaction.message(&ctx).await?;
                    handlers::starboard::handle(ctx, add_reaction.guild_id, &message).await?;
                }

                FullEvent::ReactionRemove {
                    removed_reaction, ..
                } => {
                    if removed_reaction.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = removed_reaction.message(&ctx).await?;
                    handlers::starboard::handle(ctx, removed_reaction.guild_id, &message).await?;
                }

                FullEvent::ReactionRemoveAll {
                    guild_id,
                    channel_id,
                    removed_from_message_id,
                    ..
                } => {
                    if guild_id.is_none() {
                        return Ok(());
                    }

                    let message = channel_id.message(&ctx, *removed_from_message_id).await?;
                    handlers::starboard::handle(ctx, *guild_id, &message).await?;
                }

                FullEvent::ReactionRemoveEmoji {
                    removed_reactions, ..
                } => {
                    if removed_reactions.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = removed_reactions.message(&ctx).await?;
                    handlers::starboard::handle(ctx, removed_reactions.guild_id, &message).await?;
                }

                FullEvent::GuildMemberAddition { new_member, .. } => {
                    handlers::log::member_join(ctx, new_member).await?;
                }

                FullEvent::GuildMemberRemoval {
                    user,
                    member_data_if_available,
                    guild_id,
                    ..
                } => {
                    handlers::log::member_leave(
                        ctx,
                        user,
                        member_data_if_available.as_ref(),
                        *guild_id,
                    )
                    .await?;
                }

                &_ => {}
            }

            Ok(())
        }
        .await;

        if let Err(err) = outcome {
            tracing::error!("{err:?}");
        }
    }
}
