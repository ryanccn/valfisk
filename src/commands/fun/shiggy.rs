// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{CreateReply, serenity_prelude as serenity};

use crate::{Context, http::HTTP};

#[derive(serde::Deserialize)]
struct SafebooruResponse {
    id: i64,
    source: String,
    tag_string: String,
    file_url: String,
}

/// Fetch a random shiggy
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn shiggy(
    ctx: Context<'_>,

    #[description = "Whether to send the image URL only"]
    #[flag]
    raw: bool,

    #[description = "Alternative tags to search for"] tags: Option<String>,
) -> Result<()> {
    ctx.defer().await?;

    let data: SafebooruResponse = HTTP
        .get("https://safebooru.donmai.us/posts/random.json")
        .query(&[
            (
                "tags",
                tags.as_ref().map_or("kemomimi-chan_(naga_u) naga_u", |v| v),
            ),
            ("only", "id,source,tag_string,file_url"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if raw {
        ctx.say(data.file_url).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(&[
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            format!("## [{0}](https://safebooru.donmai.us/posts/{0})", data.id),
                        )),
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            data.tag_string.replace('_', "\\_"),
                        )),
                        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                            data.source,
                        )),
                        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(false)),
                        serenity::CreateComponent::MediaGallery(serenity::CreateMediaGallery::new(
                            &[serenity::CreateMediaGalleryItem::new(
                                serenity::CreateUnfurledMediaItem::new(data.file_url),
                            )],
                        )),
                    ]),
                )]),
        )
        .await?;
    }

    Ok(())
}
