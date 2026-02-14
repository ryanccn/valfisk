// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

mod guilds;
pub mod presence;
mod say;
mod sysinfo;

use guilds::guilds;
use presence::presence;
use say::say;
use sysinfo::sysinfo;

use crate::Context;
use eyre::Result;

#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    default_member_permissions = "ADMINISTRATOR",
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("guilds", "presence", "say", "sysinfo"),
    subcommand_required
)]
pub async fn admin(ctx: Context<'_>) -> Result<()> {
    Ok(())
}
