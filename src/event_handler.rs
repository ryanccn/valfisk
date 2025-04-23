// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use crate::{config::CONFIG, handlers, storage::log::MessageLog, utils::Pluralize as _};

pub struct EventHandler;

#[serenity::async_trait]
impl serenity::EventHandler for EventHandler {
    #[expect(clippy::too_many_lines)]
    async fn dispatch(&self, ctx: &serenity::Context, event: &serenity::FullEvent) {
        use serenity::FullEvent;

        let outcome: eyre::Result<()> = async {
            match event {
                FullEvent::Ready { data_about_bot, .. } => {
                    tracing::info!("Connected to Discord as {}", data_about_bot.user.tag());

                    let commands = crate::commands::to_vec();
                    poise::builtins::register_globally(&ctx.http, &commands).await?;

                    tracing::info!(
                        "Registered {} {} ({} guild-only)",
                        commands.len(),
                        "command".pluralize(commands.len()),
                        commands.iter().filter(|c| c.guild_only).count(),
                    );

                    crate::commands::restore_presence(ctx).await?;
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
                        let logged_data = storage.get_message_log(event.message.id.get()).await?;

                        let content = event.message.content.clone();
                        let author = event.message.author.id;
                        let attachments = event.message.attachments.to_vec();

                        storage
                            .set_message_log(
                                event.message.id.get(),
                                &MessageLog::new(content.as_str(), author, attachments.clone()),
                            )
                            .await?;

                        handlers::log::edit(
                            ctx,
                            handlers::log::LogMessageIds {
                                message: event.message.id,
                                channel: event.message.channel_id,
                                guild: event.message.guild_id,
                                author: Some(author),
                            },
                            logged_data.map(|l| l.content).as_deref(),
                            content.as_ref(),
                            &attachments,
                            &timestamp,
                        )
                        .await?;
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
                            storage.get_message_log(deleted_message_id.get()).await?
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

                            storage.del_message_log(deleted_message_id.get()).await?;
                        }
                    }
                }

                FullEvent::ReactionAdd { add_reaction, .. } => {
                    if add_reaction.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = add_reaction.message(&ctx).await?;
                    handlers::starboard::handle(ctx, &message).await?;
                }

                FullEvent::ReactionRemove {
                    removed_reaction, ..
                } => {
                    if removed_reaction.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = removed_reaction.message(&ctx).await?;
                    handlers::starboard::handle(ctx, &message).await?;
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
                    handlers::starboard::handle(ctx, &message).await?;
                }

                FullEvent::ReactionRemoveEmoji {
                    removed_reactions, ..
                } => {
                    if removed_reactions.guild_id.is_none() {
                        return Ok(());
                    }

                    let message = removed_reactions.message(&ctx).await?;
                    handlers::starboard::handle(ctx, &message).await?;
                }

                FullEvent::GuildMemberAddition { new_member, .. } => {
                    handlers::log::member_join(ctx, &new_member.user).await?;
                }

                FullEvent::GuildMemberRemoval {
                    user,
                    member_data_if_available,
                    ..
                } => {
                    handlers::log::member_leave(ctx, user, member_data_if_available.as_ref())
                        .await?;
                }

                FullEvent::GuildCreate { guild, .. } => {
                    if CONFIG.guild_id.is_some_and(|id| id != guild.id) {
                        guild.id.leave(&ctx.http).await?;
                    }
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
