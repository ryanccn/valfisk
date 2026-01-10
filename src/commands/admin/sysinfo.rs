// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use std::time::Duration;
use tokio::time::sleep;

use poise::{
    CreateReply,
    serenity_prelude::{
        CreateComponent, CreateContainer, CreateContainerComponent, CreateTextDisplay, MessageFlags,
    },
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

use crate::{Context, utils};

/// Get system information for the app server
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

    let mut container = CreateContainer::new(vec![CreateContainerComponent::TextDisplay(
        CreateTextDisplay::new("### System information"),
    )])
    .accent_color(0xa78bfa);

    container = container.add_component(CreateContainerComponent::TextDisplay(
        CreateTextDisplay::new(format!(
            r"**CPU**: {} ({} cores)
**CPU usage**: {:.2}%
**Memory**: {}/{} ({:.2}%)
**Operating system**: {} ({})",
            sys.cpus().first().map_or("Unknown", |cpu| cpu.brand()),
            System::physical_core_count().unwrap_or_default(),
            sys.global_cpu_usage(),
            utils::format_bytes(sys.used_memory()),
            utils::format_bytes(sys.total_memory()),
            (sys.used_memory() as f64) / (sys.total_memory() as f64) * 100.,
            System::long_os_version().unwrap_or_else(|| "Unknown".into()),
            System::cpu_arch(),
        )),
    ));

    if let Some(proc) = sys.process(Pid::from_u32(std::process::id())) {
        container = container.add_component(CreateContainerComponent::TextDisplay(
            CreateTextDisplay::new(format!(
                r"**Process CPU usage**: {:.2}%
**Process memory**: {}
**Process uptime**: {}",
                proc.cpu_usage(),
                utils::format_bytes(proc.memory()),
                humantime::format_duration(Duration::from_secs(proc.run_time()))
            )),
        ));
    }

    if let Some(storage) = &ctx.data().storage {
        container = container.add_component(CreateContainerComponent::TextDisplay(
            CreateTextDisplay::new(format!("**KV keys**\n{}", storage.size().await?)),
        ));
    }

    ctx.send(
        CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(&[CreateComponent::Container(container)]),
    )
    .await?;

    Ok(())
}
