// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use std::time::Duration;
use tokio::time::sleep;

use poise::{CreateReply, serenity_prelude::CreateEmbed};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

use crate::{Context, utils};

/// Get system information for the bot host
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    ephemeral,
    owners_only,
    default_member_permissions = "ADMINISTRATOR",
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
#[expect(clippy::cast_precision_loss)]
pub async fn sysinfo(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let refresh_kind = RefreshKind::default()
        .with_cpu(CpuRefreshKind::default().with_cpu_usage())
        .with_memory(MemoryRefreshKind::default().with_ram())
        .with_processes(ProcessRefreshKind::default().with_cpu().with_memory());

    let mut sys = System::new();

    sys.refresh_specifics(refresh_kind);
    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    sys.refresh_specifics(refresh_kind);

    let mut embed = CreateEmbed::default()
        .title("System information")
        .color(0xa78bfa);

    embed = embed
        .field(
            "CPU",
            format!(
                "**{}** ({} cores)",
                sys.cpus().first().map_or("Unknown", |cpu| cpu.brand()),
                System::physical_core_count().unwrap_or_default()
            ),
            true,
        )
        .field("CPU usage", format!("{:.2}%", sys.global_cpu_usage()), true)
        .field(
            "Memory",
            format!(
                "{}/{} ({:.2}%)",
                utils::format_bytes(sys.used_memory()),
                utils::format_bytes(sys.total_memory()),
                (sys.used_memory() as f64) / (sys.total_memory() as f64) * 100.
            ),
            true,
        )
        .field(
            "Operating system",
            format!(
                "**{}** {} ({})",
                System::name().unwrap_or_else(|| "Unknown".into()),
                System::os_version().unwrap_or_else(|| "Unknown".into()),
                System::cpu_arch(),
            ),
            true,
        );

    if let Some(proc) = sys.process(Pid::from_u32(std::process::id())) {
        embed = embed
            .field(
                "Process CPU usage",
                format!("{:.2}%", proc.cpu_usage()),
                true,
            )
            .field("Process memory", utils::format_bytes(proc.memory()), true)
            .field(
                "Process uptime",
                humantime::format_duration(Duration::from_secs(proc.run_time())).to_string(),
                true,
            );
    }

    if let Some(storage) = &ctx.data().storage {
        embed = embed.field("KV keys", format!("{}", storage.size().await?), true);
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
