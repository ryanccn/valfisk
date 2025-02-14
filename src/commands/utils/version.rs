// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env::consts::{ARCH, OS};

use eyre::Result;
use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::Context;

/// Get version information
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only)]
pub async fn version(ctx: Context<'_>) -> Result<()> {
    let version_suffix = option_env!("CARGO_PKG_VERSION")
        .map(|v| format!(" v{v}"))
        .unwrap_or_default();

    let host = option_env!("METADATA_HOST")
        .map_or_else(|| "unknown".to_owned(), |host| format!("`{host}`"));

    let target = option_env!("METADATA_TARGET")
        .map_or_else(|| "unknown".to_owned(), |target| format!("`{target}`"));

    let last_modified = option_env!("METADATA_LAST_MODIFIED").map_or_else(
        || "unknown".to_owned(),
        |timestamp| format!("<t:{timestamp}:f>"),
    );

    let git_rev = option_env!("METADATA_GIT_REV")
        .map_or_else(|| "unknown".to_owned(), |git_rev| format!("`{git_rev}`"));

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title(format!("Valfisk{version_suffix}"))
                .field("Runtime OS", OS, true)
                .field("Runtime architecture", ARCH, true)
                .field("Target", &target, false)
                .field("Build host", &host, false)
                .field("Last modified", &last_modified, false)
                .field("Git revision", &git_rev, false)
                .color(0xf472b6),
        ),
    )
    .await?;

    Ok(())
}
