use std::env::consts::{ARCH, OS};

use color_eyre::eyre::Result;
use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::Context;

/// Get version information
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn version(ctx: Context<'_>) -> Result<()> {
    let version_suffix = match option_env!("CARGO_PKG_VERSION") {
        Some(v) => format!(" v{v}"),
        None => String::new(),
    };

    let host = match option_env!("METADATA_HOST") {
        Some(host) => format!("`{host}`"),
        None => "unknown".to_owned(),
    };

    let target = match option_env!("METADATA_TARGET") {
        Some(target) => format!("`{target}`"),
        None => "unknown".to_owned(),
    };

    let last_modified = match option_env!("METADATA_LAST_MODIFIED") {
        Some(timestamp) => format!("<t:{timestamp}:f>"),
        None => "unknown".to_owned(),
    };

    let git_rev = match option_env!("METADATA_GIT_REV") {
        Some(git_rev) => format!("`{git_rev}`"),
        None => "unknown".to_owned(),
    };

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
