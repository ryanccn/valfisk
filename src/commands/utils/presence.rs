// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::{CreateReply, serenity_prelude as serenity};

use eyre::Result;

use crate::{Context, storage::presence::PresenceKind};

/// Modify the Discord presence shown by the bot
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    default_member_permissions = "ADMINISTRATOR",
    install_context = "Guild | User"
)]
pub async fn presence(
    ctx: Context<'_>,
    #[description = "Text to display"] content: String,
    #[description = "Type of presence"] r#type: Option<PresenceKind>,
) -> Result<()> {
    let data = r#type.unwrap_or_default().with_content(&content);

    ctx.serenity_context()
        .set_presence(data.to_activity(), serenity::OnlineStatus::Online);

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
        if data.r#type == PresenceKind::Clear {
            storage.del_presence().await?;
        } else {
            storage.set_presence(&data).await?;
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn restore(ctx: &serenity::Context) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        if let Some(presence) = storage.get_presence().await? {
            ctx.set_presence(presence.to_activity(), serenity::OnlineStatus::Online);
            tracing::info!(?presence, "restored presence from storage");
        }
    }

    Ok(())
}
