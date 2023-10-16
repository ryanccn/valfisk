use anyhow::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::Context;

/// Get information on the username migration within the server
#[poise::command(slash_command)]
pub async fn pomelo(ctx: Context<'_>) -> Result<()> {
    if let Some(guild) = ctx.guild_id() {
        ctx.defer().await?;

        let members = guild
            .members(&ctx, None, None)
            .await?
            .into_iter()
            .filter(|m| !m.user.bot)
            .collect::<Vec<serenity::Member>>();

        let mut nonmigrated_users: Vec<&serenity::UserId> = vec![];

        for member in &members {
            if member.user.discriminator != std::num::NonZeroU16::new(0) {
                nonmigrated_users.push(&member.user.id);
            };
        }

        ctx.send(
            CreateReply::new().embed(
                serenity::CreateEmbed::new()
                    .title("Username migration / Pomelo")
                    .description(format!(
                        "**{}/{}** migrated",
                        nonmigrated_users.len(),
                        members.len()
                    ))
                    .field(
                        "Unmigrated users",
                        nonmigrated_users
                            .into_iter()
                            .map(|u| format!("<@{u}>"))
                            .collect::<Vec<String>>()
                            .join(" "),
                        false,
                    )
                    .color(0x2dd4bf),
            ),
        )
        .await?;
    } else {
        ctx.say("Guild unavailable").await?;
    }

    Ok(())
}
