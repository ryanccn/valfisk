// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;
use regex::Regex;

use eyre::{Result, bail};
use std::{pin::Pin, sync::LazyLock};
use tracing::warn;

use crate::{
    http::HTTP,
    utils::{serenity::suppress_embeds, truncate},
};

static GITHUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn github<'a>(captures: regex::Captures<'a>) -> Result<serenity::CreateEmbed<'static>> {
    tracing::debug!(link = &captures[0], "handling GitHub link");

    let repo = &captures["repo"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let language = file.split('.').next_back().unwrap_or_default();

    let start = captures["start"].parse::<usize>()?;
    let end = captures
        .name("end")
        .and_then(|end| end.as_str().parse::<usize>().ok());

    let lines: Vec<String> = HTTP
        .get(format!(
            "https://raw.githubusercontent.com/{repo}/{ref}/{file}"
        ))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?
        .lines()
        .map(|s| s.to_owned())
        .collect();

    let Some(selected_lines) = lines
        .get((start - 1)..(end.unwrap_or(start)))
        .map(|l| l.join("\n"))
    else {
        bail!("out of bounds line indexes");
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!(
            "{repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))
        .description(
            "```".to_owned() + language + "\n" + &truncate(&selected_lines, 2048) + "\n```",
        )
        .footer(serenity::CreateEmbedFooter::new("GitHub"))
        .timestamp(serenity::Timestamp::now());

    Ok(embed)
}

static CODEBERG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://codeberg\.org/(?P<repo>[\w\-]+/[\w.\-]+)/src/(?P<ref_type>\S+?)/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn codeberg<'a>(captures: regex::Captures<'a>) -> Result<serenity::CreateEmbed<'static>> {
    tracing::debug!(link = &captures[0], "handling Codeberg link");

    let repo = &captures["repo"];
    let ref_type = &captures["ref_type"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let language = file.split('.').next_back().unwrap_or_default();

    let start = captures["start"].parse::<usize>()?;
    let end = captures
        .name("end")
        .and_then(|end| end.as_str().parse::<usize>().ok());

    let lines: Vec<String> = HTTP
        .get(format!(
            "https://codeberg.org/{repo}/raw/{ref_type}/{ref}/{file}"
        ))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?
        .lines()
        .map(|s| s.to_owned())
        .collect();

    let Some(selected_lines) = lines
        .get((start - 1)..(end.unwrap_or(start)))
        .map(|l| l.join("\n"))
    else {
        bail!("out of bounds line indexes");
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!(
            "{repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))
        .description(
            "```".to_owned() + language + "\n" + &truncate(&selected_lines, 2048) + "\n```",
        )
        .footer(serenity::CreateEmbedFooter::new("Codeberg"))
        .timestamp(serenity::Timestamp::now());

    Ok(embed)
}

static GITLAB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://gitlab\.com/(?P<repo>[\w\-]+/[\w.\-]+)/-/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn gitlab<'a>(captures: regex::Captures<'a>) -> Result<serenity::CreateEmbed<'static>> {
    tracing::debug!(link = &captures[0], "handling GitLab link");

    let repo = &captures["repo"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let language = file.split('.').next_back().unwrap_or_default();

    let start = captures["start"].parse::<usize>()?;
    let end = captures
        .name("end")
        .and_then(|end| end.as_str().parse::<usize>().ok());

    let lines: Vec<String> = HTTP
        .get(format!("https://gitlab.com/{repo}/-/raw/{ref}/{file}"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?
        .lines()
        .map(|s| s.to_owned())
        .collect();

    let Some(selected_lines) = lines
        .get((start - 1)..(end.unwrap_or(start)))
        .map(|l| l.join("\n"))
    else {
        bail!("out of bounds line indexes");
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!(
            "{repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))
        .description(
            "```".to_owned() + language + "\n" + &truncate(&selected_lines, 2048) + "\n```",
        )
        .footer(serenity::CreateEmbedFooter::new("GitLab"))
        .timestamp(serenity::Timestamp::now());

    Ok(embed)
}

static RUST_PLAYGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

#[tracing::instrument]
async fn rust_playground<'a>(
    captures: regex::Captures<'a>,
) -> Result<serenity::CreateEmbed<'static>> {
    tracing::debug!(link = &captures[0], "handling Rust playground link");

    let gist_id = &captures["gist"];

    let gist = HTTP
        .get(format!(
            "https://gist.githubusercontent.com/rust-play/{gist_id}/raw/playground.rs"
        ))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let embed = serenity::CreateEmbed::default()
        .title(gist_id.to_owned())
        .description("```rust\n".to_owned() + &truncate(&gist, 2048) + "\n```")
        .footer(serenity::CreateEmbedFooter::new("play.rust-lang.org"))
        .timestamp(serenity::Timestamp::now())
        .color(0xdea584);

    Ok(embed)
}

static GO_PLAYGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument]
async fn go_playground<'a>(
    captures: regex::Captures<'a>,
) -> Result<serenity::CreateEmbed<'static>> {
    tracing::debug!(link = &captures[0], "handling Go playground link");

    let id = &captures["id"];

    let code = HTTP
        .get("https://go.dev/_/share")
        .query(&[("id", &id)])
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let embed = serenity::CreateEmbed::default()
        .title(id.to_owned())
        .description("```go\n".to_owned() + &truncate(&code, 2048) + "\n```")
        .footer(serenity::CreateEmbedFooter::new("go.dev/play"))
        .timestamp(serenity::Timestamp::now())
        .color(0x00b7e7);

    Ok(embed)
}

pub async fn resolve(content: &str) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds_tasks: Vec<
        Pin<Box<dyn Future<Output = Result<serenity::CreateEmbed>> + Send + Sync>>,
    > = Vec::new();

    for captures in GITHUB.captures_iter(content) {
        embeds_tasks.push(Box::pin(async move { github(captures).await }));
    }

    for captures in CODEBERG.captures_iter(content) {
        embeds_tasks.push(Box::pin(async move { codeberg(captures).await }));
    }

    for captures in GITLAB.captures_iter(content) {
        embeds_tasks.push(Box::pin(async move { gitlab(captures).await }));
    }

    for captures in RUST_PLAYGROUND.captures_iter(content) {
        embeds_tasks.push(Box::pin(async move { rust_playground(captures).await }));
    }

    for captures in GO_PLAYGROUND.captures_iter(content) {
        embeds_tasks.push(Box::pin(async move { go_playground(captures).await }));
    }

    let embeds = futures_util::future::join_all(embeds_tasks)
        .await
        .into_iter()
        .filter_map(|r| match r {
            Ok(c) => Some(c),
            Err(err) => {
                warn!("{err:?}");
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(embeds)
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if message
        .flags
        .is_some_and(|f| f.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
    {
        return Ok(());
    }

    let embeds = resolve(&message.content).await?;

    if !embeds.is_empty() {
        suppress_embeds(ctx, message).await?;

        message
            .channel_id
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embeds(embeds)
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::default().replied_user(false),
                    )
                    .reference_message(message),
            )
            .await?;
    }

    Ok(())
}
