// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use crate::{config::CONFIG, handlers, storage::log::MessageLog, utils::Pluralize as _};

pub struct EventHandler;

macro_rules! wrap_event_handler {
    ($fn: expr) => {{
        let outcome: ::eyre::Result<()> = ($fn)().await;

        if let Err(err) = outcome {
            ::tracing::error!("{err:?}");
        }
    }};
}

#[serenity::async_trait]
impl serenity::EventHandler for EventHandler {
    async fn ready(&self, ctx: serenity::Context, ready: serenity::Ready) {
        wrap_event_handler!(async || {
            tracing::info!("Connected to Discord as {}", ready.user.tag());

            let commands = crate::commands::to_vec();
            poise::builtins::register_globally(&ctx.http, &commands).await?;

            tracing::info!(
                "Registered {} {} ({} guild-only)",
                commands.len(),
                "command".pluralize(commands.len()),
                commands.iter().filter(|c| c.guild_only).count(),
            );

            crate::commands::restore_presence(&ctx).await?;
            Ok(())
        });
    }

    async fn message(&self, ctx: serenity::Context, message: serenity::Message) {
        wrap_event_handler!(async || {
            if message.guild_id.is_some() {
                Box::pin(handlers::message_guild(&ctx, &message)).await?;
            } else {
                handlers::message_dm(&ctx, &message).await?;
            }

            Ok(())
        });
    }

    async fn message_update(
        &self,
        ctx: serenity::Context,
        _old_if_available: Option<serenity::Message>,
        event: serenity::MessageUpdateEvent,
    ) {
        wrap_event_handler!(async || {
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
                    &ctx,
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

            Ok(())
        });
    }

    async fn message_delete(
        &self,
        ctx: serenity::Context,
        channel_id: serenity::ChannelId,
        deleted_message_id: serenity::MessageId,
        guild_id: Option<serenity::GuildId>,
    ) {
        wrap_event_handler!(async || {
            if guild_id.is_none() {
                return Ok(());
            }

            handlers::starboard::handle_deletion(&ctx, deleted_message_id, channel_id, guild_id)
                .await?;

            let timestamp = serenity::Timestamp::now();

            if let Some(storage) = &ctx.data::<crate::Data>().storage {
                if let Some(logged_data) = storage.get_message_log(deleted_message_id.get()).await?
                {
                    handlers::log::delete(
                        &ctx,
                        handlers::log::LogMessageIds {
                            message: deleted_message_id,
                            channel: channel_id,
                            guild: guild_id,
                            author: Some(logged_data.author),
                        },
                        &logged_data,
                        &timestamp,
                    )
                    .await?;

                    storage.del_message_log(deleted_message_id.get()).await?;
                }
            }

            Ok(())
        });
    }

    async fn reaction_add(&self, ctx: serenity::Context, add_reaction: serenity::Reaction) {
        wrap_event_handler!(async || {
            if add_reaction.guild_id.is_none() {
                return Ok(());
            }

            let message = add_reaction.message(&ctx).await?;
            handlers::starboard::handle(&ctx, &message).await?;

            Ok(())
        });
    }

    async fn reaction_remove(&self, ctx: serenity::Context, removed_reaction: serenity::Reaction) {
        wrap_event_handler!(async || {
            if removed_reaction.guild_id.is_none() {
                return Ok(());
            }

            let message = removed_reaction.message(&ctx).await?;
            handlers::starboard::handle(&ctx, &message).await?;

            Ok(())
        });
    }

    async fn reaction_remove_all(
        &self,
        ctx: serenity::Context,
        guild_id: Option<serenity::GuildId>,
        channel_id: serenity::ChannelId,
        message_id: serenity::MessageId,
    ) {
        wrap_event_handler!(async || {
            if guild_id.is_none() {
                return Ok(());
            }

            let message = channel_id.message(&ctx, message_id).await?;
            handlers::starboard::handle(&ctx, &message).await?;

            Ok(())
        });
    }

    async fn reaction_remove_emoji(
        &self,
        ctx: serenity::Context,
        removed_reactions: serenity::Reaction,
    ) {
        wrap_event_handler!(async || {
            if removed_reactions.guild_id.is_none() {
                return Ok(());
            }

            let message = removed_reactions.message(&ctx).await?;
            handlers::starboard::handle(&ctx, &message).await?;

            Ok(())
        });
    }

    async fn guild_member_addition(&self, ctx: serenity::Context, new_member: serenity::Member) {
        wrap_event_handler!(async || {
            handlers::log::member_join(&ctx, &new_member.user).await?;
            Ok(())
        });
    }

    async fn guild_member_removal(
        &self,
        ctx: serenity::Context,
        _guild_id: serenity::GuildId,
        user: serenity::User,
        member: Option<serenity::Member>,
    ) {
        wrap_event_handler!(async || {
            handlers::log::member_leave(&ctx, &user, member.as_ref()).await?;
            Ok(())
        });
    }

    async fn guild_create(
        &self,
        ctx: serenity::Context,
        guild: serenity::Guild,
        _is_new: Option<bool>,
    ) {
        wrap_event_handler!(async || {
            if CONFIG.guild_id.is_some_and(|id| id != guild.id) {
                guild.id.leave(&ctx.http).await?;
            }

            Ok(())
        });
    }
}
