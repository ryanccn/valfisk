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
) -> Result<()> {
    ctx.defer().await?;

    let data: SafebooruResponse = HTTP
        .get("https://safebooru.donmai.us/posts/random.json")
        .query(&[
            ("tags", "kemomimi-chan_(naga_u) naga_u"),
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
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title(data.id.to_string())
                    .field("Tags", data.tag_string.replace('_', "\\_"), false)
                    .field("Source", &data.source, false)
                    .url(format!("https://safebooru.donmai.us/posts/{}", data.id))
                    .image(&data.file_url)
                    .color(0xfef9c3),
            ),
        )
        .await?;
    }

    Ok(())
}
