// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::borrow::Cow;

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::Context;

fn short_link<'a>(url: impl Into<Cow<'a, str>>) -> String {
    let url: Cow<'a, str> = url.into();
    format!(
        "[{}]({})",
        url::Url::parse(&url)
            .ok()
            .and_then(|u| u.domain().map(|s| s.to_owned()))
            .unwrap_or_else(|| "view".to_owned()),
        url
    )
}

/// Show information about a user
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn user(ctx: Context<'_>, user: serenity::UserId) -> Result<()> {
    ctx.defer().await?;

    let user = user.to_user(&ctx).await?;

    let mut container =
        serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::Section(
            serenity::CreateSection::new(
                vec![
                    serenity::CreateSectionComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "## {}",
                            user.global_name.as_ref().unwrap_or(&user.name)
                        )),
                    ),
                    serenity::CreateSectionComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(format!(
                            "### @{}
**ID**: {}
**Avatar**: {}
**Banner**: {}
**Accent color**: {}
**Flags**: {}
**Created at**: <t:{}:F> (<t:{}:R>)",
                            user.tag(),
                            user.id,
                            short_link(user.face()),
                            user.banner_url()
                                .map_or_else(|| "*None*".to_owned(), |u| short_link(&u)),
                            user.accent_colour
                                .map_or("*None*".to_owned(), |c| format!("#{}", c.hex())),
                            if user.flags.is_empty() {
                                "*None*".to_owned()
                            } else {
                                user.flags
                                    .iter_names()
                                    .map(|(n, _)| format!("`{n}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            },
                            user.id.created_at().timestamp(),
                            user.id.created_at().timestamp()
                        )),
                    ),
                ],
                serenity::CreateSectionAccessory::Thumbnail(serenity::CreateThumbnail::new(
                    serenity::CreateUnfurledMediaItem::new(user.face()),
                )),
            ),
        )]);

    if let Some(color) = &user.accent_colour {
        container = container.accent_color(*color);
    }

    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .allowed_mentions(serenity::CreateAllowedMentions::new())
            .components(&[serenity::CreateComponent::Container(container)]),
    )
    .await?;

    Ok(())
}
