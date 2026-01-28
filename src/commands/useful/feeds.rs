// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::sync::LazyLock;

use eyre::Result;
use poise::{CreateReply, serenity_prelude as serenity};
use regex::Regex;

use crate::{Context, http::HTTP};

static RSS_TITLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<title>(?P<title>.+?)</title>").unwrap());

static RSS_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"<link>(?P<link>https?://[-a-zA-Z0-9@:%._\+~#=]+\.[a-zA-Z0-9()]+\b[-a-zA-Z0-9()@:%_\+.~#?&//=]*)</link>"
    ).unwrap()
});

async fn feed(ctx: Context<'_>, name: &str, color: u32, feed: &str) -> Result<()> {
    let feed = HTTP
        .get(feed)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let titles = RSS_TITLE_REGEX
        .captures_iter(&feed)
        .filter_map(|c| c.name("title").map(|m| m.as_str()))
        .map(|s| {
            s.strip_prefix("<![CDATA[")
                .map_or(s, |s| s.strip_suffix("]]>").unwrap_or(s))
        });

    let links = RSS_LINK_REGEX
        .captures_iter(&feed)
        .filter_map(|c| c.name("link").map(|m| m.as_str()));

    let data = titles.zip(links).skip(1).take(10).collect::<Vec<_>>();

    ctx.send(
        CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(&[serenity::CreateComponent::Container(
                serenity::CreateContainer::new(&[serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!(
                        "## {name}\n{}\n\n{}",
                        serenity::FormattedTimestamp::now(),
                        data.iter()
                            .map(|(t, l)| format!("**{t}**\n{l}"))
                            .collect::<Vec<_>>()
                            .join("\n\n"),
                    )),
                )])
                .accent_color(color),
            )]),
    )
    .await?;

    Ok(())
}

/// Show front page posts on Lobsters
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn lobsters(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    feed(ctx, "Lobste.rs", 0xAC130D, "https://lobste.rs/rss").await?;
    Ok(())
}

/// Show front page posts on Hacker News
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    rename = "hacker-news",
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn hacker_news(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    feed(ctx, "Hacker News", 0xFF6600, "https://hnrss.org/frontpage").await?;
    Ok(())
}
