use color_eyre::eyre::Result;
use poise::{
    serenity_prelude::{futures::StreamExt, ChannelId, CreateEmbed},
    CreateReply,
};

use crate::{reqwest_client::HTTP, template_channel::Config as TemplateChannelConfig, Context};

/// Apply a channel template from a URL to a channel
#[poise::command(rename = "template-channel", slash_command, guild_only, ephemeral)]
pub async fn template_channel(
    ctx: Context<'_>,
    #[description = "The URL to fetch the template from"] url: String,
    #[description = "The channel to apply the template to"] channel: ChannelId,
    #[description = "Whether or not to clear the channel (default true)"] clear: Option<bool>,
) -> Result<()> {
    let clear = clear.unwrap_or(true);
    ctx.defer_ephemeral().await?;

    let source = HTTP.get(&url).send().await?.text().await?;
    let data = TemplateChannelConfig::parse(&source)?;
    let messages = data.to_messages();

    if clear {
        let mut message_iter = channel.messages_iter(&ctx).boxed();
        while let Some(message) = message_iter.next().await {
            if let Ok(message) = message {
                message.delete(&ctx).await?;
            }
        }
    }

    for m in messages {
        channel.send_message(&ctx, m).await?;
    }

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Applied channel template!")
                .field("URL", format!("`{url}`"), false)
                .field("Channel", format!("<#{channel}>"), false)
                .field("Components", format!("{}", data.components.len()), false)
                .color(0x22d3ee),
        ),
    )
    .await?;

    Ok(())
}
