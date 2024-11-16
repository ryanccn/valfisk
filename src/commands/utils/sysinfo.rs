// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use std::time::Duration;
use tokio::time::sleep;

use poise::{serenity_prelude::CreateEmbed, CreateReply};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

use crate::Context;

/// Get system information for the bot host
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only)]
#[allow(clippy::cast_precision_loss)]
pub async fn sysinfo(ctx: Context<'_>) -> Result<()> {
    let refresh_kind = RefreshKind::new()
        .with_cpu(CpuRefreshKind::new().with_cpu_usage())
        .with_memory(MemoryRefreshKind::new().with_ram())
        .with_processes(ProcessRefreshKind::new().with_cpu().with_memory());

    let mut sys = System::new();

    sys.refresh_specifics(refresh_kind);
    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    sys.refresh_specifics(refresh_kind);

    let mut embed = CreateEmbed::default()
        .title("System information")
        .color(0xa78bfa);

    embed = embed.field(
        "CPU",
        format!(
            "**{}** ({} cores)",
            sys.cpus().first().map_or("Unknown", |cpu| cpu.brand()),
            sys.physical_core_count().unwrap_or_default()
        ),
        true,
    );

    embed = embed.field("CPU usage", format!("{:.2}%", sys.global_cpu_usage()), true);

    embed = embed.field(
        "Memory",
        format!(
            "{}/{} ({:.2}%)",
            bytesize::to_string(sys.used_memory(), true),
            bytesize::to_string(sys.total_memory(), true),
            (sys.used_memory() as f64) / (sys.total_memory() as f64) * 100.
        ),
        true,
    );

    embed = embed.field(
        "Operating system",
        format!(
            "**{}** {}{}",
            System::name().unwrap_or_else(|| "Unknown".into()),
            System::os_version().unwrap_or_else(|| "Unknown".into()),
            System::cpu_arch()
                .map(|arch| format!(" ({arch})"))
                .unwrap_or_default(),
        ),
        true,
    );

    if let Some(proc) = sys.process(Pid::from_u32(std::process::id())) {
        embed = embed.field(
            "Process CPU usage",
            format!("{:.2}%", proc.cpu_usage()),
            true,
        );

        embed = embed.field(
            "Process memory",
            bytesize::to_string(proc.memory(), true),
            true,
        );

        embed = embed.field(
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
