// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;
use regex::Regex;

use eyre::{Result, eyre};
use std::sync::LazyLock;
use tokio::task::JoinSet;

use crate::{
    http,
    utils::{serenity::suppress_embeds, truncate},
};

static GITHUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn github<'a, 'b>(m: &'a str) -> Result<serenity::CreateEmbed<'b>> {
    let captures = GITHUB
        .captures(m)
        .ok_or_else(|| eyre!("could not obtain captures"))?;

    tracing::debug!("Handling GitHub link {}", &captures[0]);

    let repo = &captures["repo"];
    let ref_ = &captures["ref"];
    let file = &captures["file"];

    let language = file.split('.').last().unwrap_or_default();

    let start = captures["start"].parse::<usize>()?;
    let end = captures
        .name("end")
        .and_then(|end| end.as_str().parse::<usize>().ok());

    let lines: Vec<String> = http::HTTP
        .get(format!(
            "https://raw.githubusercontent.com/{repo}/{ref_}/{file}"
        ))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?
        .lines()
        .map(|s| s.to_owned())
        .collect();

    let selected_lines = lines[(start - 1)..(end.unwrap_or(start))].join("\n");

    let embed = serenity::CreateEmbed::default()
        .title(format!(
            "{repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))
        .description(
            "```".to_owned() + language + "\n" + &truncate(&selected_lines, 2048) + "\n```",
        )
        .footer(serenity::CreateEmbedFooter::new(ref_.to_owned()))
        .timestamp(serenity::Timestamp::now());

    Ok(embed)
}

static RUST_PLAYGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

#[tracing::instrument]
async fn rust_playground<'a, 'b>(m: &'a str) -> Result<serenity::CreateEmbed<'b>> {
    let captures = RUST_PLAYGROUND
        .captures(m)
        .ok_or_else(|| eyre!("could not obtain captures"))?;

    tracing::debug!("Handling Rust playground link {}", &captures[0]);

    let gist_id = &captures["gist"];

    let gist = http::HTTP
        .get(format!(
            "https://gist.githubusercontent.com/rust-play/{gist_id}/raw/playground.rs"
        ))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let embed = serenity::CreateEmbed::default()
        .title("Rust Playground")
        .description("```rust\n".to_owned() + &truncate(&gist, 2048) + "\n```")
        .footer(serenity::CreateEmbedFooter::new(gist_id.to_owned()))
        .timestamp(serenity::Timestamp::now())
        .color(0xdea584);

    Ok(embed)
}

static GO_PLAYGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument]
async fn go_playground<'a, 'b>(m: &'a str) -> Result<serenity::CreateEmbed<'b>> {
    let captures = GO_PLAYGROUND
        .captures(m)
        .ok_or_else(|| eyre!("could not obtain captures"))?;

    tracing::debug!("Handling Go playground link {}", &captures[0]);

    let id = &captures["id"];

    let code = http::HTTP
        .get("https://go.dev/_/share")
        .query(&[("id", &id)])
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let embed = serenity::CreateEmbed::default()
        .title("Go Playground")
        .description("```go\n".to_owned() + &truncate(&code, 2048) + "\n```")
        .footer(serenity::CreateEmbedFooter::new(id.to_owned()))
        .timestamp(serenity::Timestamp::now())
        .color(0x00b7e7);

    Ok(embed)
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    let mut embeds_tasks: JoinSet<Result<serenity::CreateEmbed>> = JoinSet::new();

    for m in GITHUB.find_iter(&message.content) {
        let m = m.as_str().to_owned();
        embeds_tasks.spawn(async move { github(&m).await });
    }

    for m in RUST_PLAYGROUND.find_iter(&message.content) {
        let m = m.as_str().to_owned();
        embeds_tasks.spawn(async move { rust_playground(&m).await });
    }

    for m in GO_PLAYGROUND.find_iter(&message.content) {
        let m = m.as_str().to_owned();
        embeds_tasks.spawn(async move { go_playground(&m).await });
    }

    let embeds = embeds_tasks
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

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
    };

    Ok(())
}
