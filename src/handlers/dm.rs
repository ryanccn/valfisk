use poise::serenity_prelude as serenity;

use eyre::Result;
use humansize::{format_size, FormatSizeOptions};
use once_cell::sync::Lazy;

static DM_LOGS_CHANNEL: Lazy<Option<serenity::ChannelId>> = Lazy::new(|| {
    std::env::var("DM_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<serenity::ChannelId>().ok())
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if message.channel(&ctx).await?.private().is_some() {
        if let Some(logs_channel) = *DM_LOGS_CHANNEL {
            let mut embed = serenity::CreateEmbed::default()
                .description(message.content.clone())
                .author(
                    serenity::CreateEmbedAuthor::new(message.author.tag())
                        .icon_url(message.author.face()),
                )
                .color(0x9775fa)
                .timestamp(message.timestamp);

            if !message.attachments.is_empty() {
                embed = embed.field(
                    "Attachments",
                    message
                        .attachments
                        .iter()
                        .map(|att| {
                            format!(
                                "[{}]({}) ({})",
                                att.filename,
                                att.url,
                                format_size(
                                    att.size,
                                    FormatSizeOptions::default().space_after_value(true)
                                )
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    false,
                );
            }

            logs_channel
                .send_message(&ctx.http, serenity::CreateMessage::default().embed(embed))
                .await?;

            message.react(&ctx.http, '✅').await?;
        }
    }

    Ok(())
}
