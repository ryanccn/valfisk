use poise::{serenity_prelude::CreateEmbed, CreateReply};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::Context;
use color_eyre::eyre::Result;

/// Get system information for the bot host
#[poise::command(slash_command, guild_only)]
#[allow(clippy::cast_precision_loss)]
pub async fn sysinfo(ctx: Context<'_>) -> Result<()> {
    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
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

    embed = embed.field(
        "CPU load",
        format!("{:.2}%", sys.global_cpu_info().cpu_usage()),
        false,
    );

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
            "**{}** {} ({})",
            os.os_type(),
            os.version(),
            os.architecture().unwrap_or_default(),
        ),
        false,
    );

    if let Some(redis) = &ctx.data().redis {
        let mut conn = redis.get_async_connection().await?;
        let keys: u64 = redis::cmd("DBSIZE").query_async(&mut conn).await?;

        embed = embed.field("KV keys", format!("{keys}"), false);
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
