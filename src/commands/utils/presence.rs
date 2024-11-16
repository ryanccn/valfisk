// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::{serenity_prelude as serenity, CreateReply};

use eyre::Result;
use tracing::info;

use crate::{storage::presence::PresenceChoice, Context};

/// Modify the Discord presence shown by the bot
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    guild_only,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn presence(
    ctx: Context<'_>,
    #[description = "Text to display"] content: String,
    #[description = "Type of presence"] r#type: Option<PresenceChoice>,
) -> Result<()> {
    let data = r#type.unwrap_or_default().to_data(&content);

    ctx.serenity_context()
        .set_presence(Some(data.to_activity()), serenity::OnlineStatus::Online);

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Presence set!")
                .field("Type", data.r#type.to_string(), false)
                .field("Content", &data.content, false)
                .color(0x4ade80),
        ),
    )
    .await?;

    if let Some(storage) = &ctx.data().storage {
        storage.set_presence(&data).await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn restore(ctx: &serenity::Context, data: &crate::Data) -> Result<()> {
    if let Some(storage) = &data.storage {
        if let Some(presence) = storage.get_presence().await? {
            ctx.set_presence(Some(presence.to_activity()), serenity::OnlineStatus::Online);

            info!("Restored presence from Redis");
        }
    }

    Ok(())
}
