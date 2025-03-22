// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env::consts::{ARCH, OS};

use eyre::Result;
use poise::{CreateReply, serenity_prelude::CreateEmbed};

use crate::Context;

/// Get version information
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only)]
pub async fn version(ctx: Context<'_>) -> Result<()> {
    let version_suffix = option_env!("CARGO_PKG_VERSION")
        .map(|v| format!(" v{v}"))
        .unwrap_or_default();

    let target = option_env!("METADATA_TARGET")
        .map_or_else(|| "*Unknown*".to_owned(), |target| format!("`{target}`"));

    let host = option_env!("METADATA_HOST")
        .map_or_else(|| "*Unknown*".to_owned(), |host| format!("`{host}`"));

    let last_modified = option_env!("METADATA_LAST_MODIFIED").map_or_else(
        || "*Unknown*".to_owned(),
        |timestamp| format!("<t:{timestamp}:f>"),
    );

    let revision = option_env!("METADATA_REVISION").map_or_else(
        || "*Unknown*".to_owned(),
        |rev| format!("[`{rev}`](https://github.com/ryanccn/valfisk/tree/{rev})",),
    );

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title(format!("Valfisk{version_suffix}"))
                .field("Runtime OS", format!("{ARCH}-{OS}"), true)
                .field("Target", &target, false)
                .field("Build host", &host, false)
                .field("Last modified", &last_modified, false)
                .field("Revision", &revision, false)
                .color(0xf472b6),
        ),
    )
    .await?;

    Ok(())
}
