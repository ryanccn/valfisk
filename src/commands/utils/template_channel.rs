// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{CreateEmbed, GenericChannelId, Mentionable as _, futures::StreamExt},
};

use crate::{Context, http::HTTP, template_channel::Template};

/// Apply a channel template from a URL to a channel
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
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
    #[description = "The URL to fetch the template from"] url: String,
    #[description = "The channel to apply the template to"]
    #[channel_types("Text")]
    channel: GenericChannelId,
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

    let data = Template::parse(&source)?;
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
                .title("Applied channel template")
                .field("URL", format!("`{url}`"), false)
                .field("Channel", channel.mention().to_string(), false)
                .field("Components", data.components.len().to_string(), false)
                .color(0x22d3ee),
        ),
    )
    .await?;

    Ok(())
}
