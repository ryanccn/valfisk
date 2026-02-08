// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};

use poise::serenity_prelude as serenity;

use crate::{Context, utils};

/// Time yourself out for a specific duration
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    rename = "self-timeout",
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn self_timeout(
    ctx: Context<'_>,
    #[description = "The duration to time yourself out for"] duration: String,
    #[description = "The reason for the timeout"] reason: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    if let Ok(duration) = humantime::parse_duration(&duration) {
        let mut member = ctx
            .author_member()
            .await
            .ok_or_else(|| eyre!("could not find member for author"))?;

        let start = chrono::Utc::now();
        let end = start + duration;

        let (confirmed, reply) = utils::serenity::interaction_confirm(
            &ctx,
            serenity::CreateContainer::new(&[
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    "### Requesting self-timeout",
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!("**Start**\n<t:{0}:F>", start.timestamp()),
                )),
                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    format!("**End**\n<t:{0}:F> (<t:{0}:R>)", end.timestamp()),
                )),
            ])
            .accent_color(0xffd43b),
        )
        .await?;

        if confirmed {
            member
                .to_mut()
                .edit(
                    ctx.http(),
                    serenity::EditMember::default()
                        .disable_communication_until(end.into())
                        .audit_log_reason(&format!(
                            "Requested self timeout{}",
                            reason
                                .as_ref()
                                .map(|r| format!(": {r}"))
                                .unwrap_or_default()
                        )),
                )
                .await?;

            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .components(&[serenity::CreateComponent::Container(
                            serenity::CreateContainer::new(&[
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new("### Self-timeout in effect"),
                                ),
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new(format!(
                                        "**Start**\n<t:{0}:F>",
                                        start.timestamp()
                                    )),
                                ),
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new(format!(
                                        "**End**\n<t:{0}:F> (<t:{0}:R>)",
                                        end.timestamp()
                                    )),
                                ),
                            ])
                            .accent_color(0x4ade80),
                        )]),
                )
                .await?;
        } else {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .components(&[serenity::CreateComponent::Container(
                            serenity::CreateContainer::new(&[
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new("### Self-timeout cancelled"),
                                ),
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new(format!(
                                        "**Start**\n<t:{0}:F>",
                                        start.timestamp()
                                    )),
                                ),
                                serenity::CreateContainerComponent::TextDisplay(
                                    serenity::CreateTextDisplay::new(format!(
                                        "**End**\n<t:{0}:F> (<t:{0}:R>)",
                                        end.timestamp()
                                    )),
                                ),
                            ])
                            .accent_color(0xff6b6b),
                        )]),
                )
                .await?;
        }
    } else {
        ctx.say("Invalid duration!").await?;
    }

    Ok(())
}
