// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;
use regex::Regex;

use eyre::Result;
use std::sync::LazyLock;
use tracing::debug;

use crate::{config::CONFIG, reqwest_client, utils::serenity::suppress_embeds};

static GITHUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn github(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in GITHUB.captures_iter(&message.content) {
        debug!(
            "Handling GitHub link {} on message {}",
            &captures[0], message.id
        );

        let repo = &captures["repo"];
        let ref_ = &captures["ref"];
        let file = &captures["file"];

        let language = file.split('.').last().unwrap_or("");

        let start = captures["start"].parse::<usize>()?;
        let end = captures
            .name("end")
            .and_then(|end| end.as_str().parse::<usize>().ok());

        let lines: Vec<String> = reqwest_client::HTTP
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

        let idx_start = start - 1;
        let idx_end = end.unwrap_or(start);

        let selected_lines = &lines[idx_start..idx_end];

        let embed = serenity::CreateEmbed::default()
            .title(format!(
                "{repo} {file} L{start}{}",
                end.map_or_else(String::new, |end| format!("-{end}"))
            ))
            .description("```".to_owned() + language + "\n" + &selected_lines.join("\n") + "\n```")
            .footer(serenity::CreateEmbedFooter::new(ref_.to_owned()))
            .timestamp(serenity::Timestamp::now());

        embeds.push(embed);
    }

    Ok(embeds)
}

static RUST_PLAYGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn rust_playground(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in RUST_PLAYGROUND.captures_iter(&message.content) {
        debug!(
            "Handling Rust Playground link {} on message {}",
            &captures[0], message.id
        );

        let gist_id = &captures["gist"];

        let gist = reqwest_client::HTTP
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
            .description("```rust\n".to_owned() + &gist + "\n```")
            .footer(serenity::CreateEmbedFooter::new(gist_id.to_owned()))
            .timestamp(serenity::Timestamp::now())
            .color(0xdea584);

        embeds.push(embed);
    }

    Ok(embeds)
}

static GO_PLAYGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn go_playground(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in GO_PLAYGROUND.captures_iter(&message.content) {
        debug!(
            "Handling Go Playground link {} on message {}",
            &captures[0], message.id
        );

        let id = &captures["id"];

        let code = reqwest_client::HTTP
            .get("https://go.dev/_/share")
            .query(&[("id", &id)])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let embed = serenity::CreateEmbed::default()
            .title("Go Playground")
            .description("```go\n".to_owned() + &code + "\n```")
            .footer(serenity::CreateEmbedFooter::new(id.to_owned()))
            .timestamp(serenity::Timestamp::now())
            .color(0x00b7e7);

        embeds.push(embed);
    }

    Ok(embeds)
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.guild_id != CONFIG.guild_id {
        return Ok(());
    }

    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    let (one, two, three) = tokio::try_join!(
        github(message),
        rust_playground(message),
        go_playground(message),
    )?;

    embeds.extend(one);
    embeds.extend(two);
    embeds.extend(three);

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
