// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env::consts::{ARCH, OS};

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{CreateComponent, CreateContainer, CreateTextDisplay, MessageFlags},
};

use crate::Context;

/// Get version information
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn version(ctx: Context<'_>) -> Result<()> {
    let version_suffix = option_env!("CARGO_PKG_VERSION")
        .map(|v| format!(" v{v}"))
        .unwrap_or_default();

    let target = option_env!("METADATA_TARGET")
        .map_or_else(|| "*Unknown*".to_owned(), |target| format!("`{target}`"));

    let host = option_env!("METADATA_HOST")
        .map_or_else(|| "*Unknown*".to_owned(), |host| format!("`{host}`"));

    let revision = option_env!("METADATA_REVISION").map_or_else(
        || "*Unknown*".to_owned(),
        |rev| format!("[`{rev}`](https://github.com/ryanccn/valfisk/tree/{rev})"),
    );

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(CreateContainer::new(&[
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "## Valfisk{version_suffix}"
                ))),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "**Runtime OS**\n{ARCH}-{OS}"
                ))),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "**Build target**\n{target}"
                ))),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "**Build host**\n{host}"
                ))),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "**Revision**\n{revision}"
                ))),
            ]))]),
    )
    .await?;

    Ok(())
}
