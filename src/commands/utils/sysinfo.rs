use poise::{serenity_prelude::CreateEmbed, CreateReply};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::Context;
use color_eyre::eyre::Result;

/// Get system information for the bot host
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[allow(clippy::cast_precision_loss)]
pub async fn sysinfo(ctx: Context<'_>) -> Result<()> {
    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_memory(MemoryRefreshKind::everything().with_ram()),
    );
    let os = os_info::get();

    let mut embed = CreateEmbed::default()
        .title("System information")
        .color(0xa78bfa);

    embed = embed.field(
        "CPU",
        format!(
            "**{}** ({} cores)",
            match sys.cpus().first() {
                Some(cpu) => cpu.brand(),
                None => "Unknown",
            },
            sys.physical_core_count().unwrap_or_default()
        ),
        false,
    );

    embed = embed.field("CPU load", format!("{:.2}%", sys.global_cpu_usage()), false);

    embed = embed.field(
        "Memory",
        format!(
            "{}/{} ({:.2}%)",
            bytesize::to_string(sys.used_memory(), true),
            bytesize::to_string(sys.total_memory(), true),
            (sys.used_memory() as f64) / (sys.total_memory() as f64) * 100.
        ),
        false,
    );

    embed = embed.field(
        "Operating system",
        format!(
            "**{}** {}{}",
            os.os_type(),
            os.version(),
            match os.architecture() {
                Some(arch) => format!(" ({arch})"),
                None => String::new(),
            },
        ),
        false,
    );

    if let Some(storage) = &ctx.data().storage {
        embed = embed.field("KV keys", format!("{}", storage.size().await?), false);
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
