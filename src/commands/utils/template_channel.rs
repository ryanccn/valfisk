// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    serenity_prelude::{futures::StreamExt, ChannelId, CreateEmbed, Mentionable as _},
    CreateReply,
};

use crate::{reqwest_client::HTTP, template_channel::Config as TemplateChannelConfig, Context};

/// Apply a channel template from a URL to a channel
#[poise::command(
    rename = "template-channel",
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn template_channel(
    ctx: Context<'_>,
    #[description = "The URL to fetch the template from"] url: String,
    #[description = "The channel to apply the template to"] channel: ChannelId,
    #[description = "Whether or not to clear the channel (default true)"] clear: Option<bool>,
) -> Result<()> {
    let clear = clear.unwrap_or(true);
    ctx.defer_ephemeral().await?;

    let source = HTTP
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let data = TemplateChannelConfig::parse(&source)?;
    let messages = data.to_messages();

    if clear {
        let mut message_iter = channel.messages_iter(&ctx).boxed();
        while let Some(message) = message_iter.next().await {
            if let Ok(message) = message {
                message.delete(ctx.http(), None).await?;
            }
        }
    }

    for m in messages {
        channel.send_message(ctx.http(), m).await?;
    }

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Applied channel template!")
                .field("URL", format!("`{url}`"), false)
                .field("Channel", channel.mention().to_string(), false)
                .field("Components", data.components.len().to_string(), false)
                .color(0x22d3ee),
        ),
    )
    .await?;

    Ok(())
}
