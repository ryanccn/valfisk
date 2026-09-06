// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{
        Attachment, CreateComponent, CreateContainer, CreateContainerComponent, CreateTextDisplay,
        GenericChannelId, Mentionable as _, Message, MessageFlags, futures::StreamExt as _,
    },
};

use crate::{Context, template_channel::Template};

/// Bulk-deletes messages where possible, falling back to individual deletion for
/// messages older than Discord's 14-day bulk-delete cutoff. Failures are logged and
/// skipped rather than aborting the whole clear operation.
async fn delete_batch(ctx: Context<'_>, channel: GenericChannelId, batch: Vec<Message>) {
    let (bulk_eligible, too_old): (Vec<_>, Vec<_>) = batch
        .into_iter()
        .partition(|m| *m.timestamp >= chrono::Utc::now() - chrono::Duration::weeks(2));

    if bulk_eligible.len() >= 2 {
        let ids = bulk_eligible.iter().map(|m| m.id).collect::<Vec<_>>();
        if let Err(err) = channel.delete_messages(ctx.http(), &ids, None).await {
            tracing::warn!("{err:?}");
        }
    } else {
        for message in bulk_eligible {
            if let Err(err) = message.delete(ctx.http(), None).await {
                tracing::warn!("{err:?}");
            }
        }
    }

    for message in too_old {
        if let Err(err) = message.delete(ctx.http(), None).await {
            tracing::warn!("{err:?}");
        }
    }
}

/// Apply a channel template from a file to a channel
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    rename = "template-channel",
    slash_command,
    ephemeral,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn template_channel(
    ctx: Context<'_>,
    #[description = "The channel to apply the template to"]
    #[channel_types("Text")]
    channel: GenericChannelId,
    #[description = "The file to parse the template from"] attachment: Attachment,
    #[description = "Whether or not to clear the channel (default: false)"] clear: Option<bool>,
) -> Result<()> {
    let clear = clear.unwrap_or(false);
    ctx.defer_ephemeral().await?;

    if !attachment.filename.ends_with(".toml") {
        ctx.say("Attachment is not a TOML file!").await?;
        return Ok(());
    }

    if attachment.size > 1_000_100 {
        ctx.say("Attachment too large!").await?;
        return Ok(());
    }

    let source = attachment.download().await?;
    let source = String::from_utf8_lossy(&source);

    let data = Template::parse(&source)?;
    let messages = data.to_messages();

    if clear {
        let mut message_iter = channel.messages_iter(&ctx).boxed();
        let mut batch = Vec::new();

        while let Some(message) = message_iter.next().await {
            if let Ok(message) = message {
                batch.push(message);
            }

            if batch.len() >= 100 {
                delete_batch(ctx, channel, std::mem::take(&mut batch)).await;
            }
        }

        if !batch.is_empty() {
            delete_batch(ctx, channel, batch).await;
        }
    }

    for message in messages {
        channel.send_message(ctx.http(), message).await?;
    }

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(
                CreateContainer::new(&[
                    CreateContainerComponent::TextDisplay(CreateTextDisplay::new(
                        "### Applied channel template",
                    )),
                    CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                        "`{}` → {} (*{} components*)",
                        attachment.filename,
                        channel.mention(),
                        data.components.len()
                    ))),
                ])
                .accent_color(0x22d3ee),
            )]),
    )
    .await?;

    Ok(())
}
