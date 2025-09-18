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

fn dedent(source: &str) -> String {
    let mut cur_indent: Option<String> = None;

    for line in source.lines().filter(|l| !l.trim().is_empty()) {
        let whitespace = line
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();

        cur_indent = if cur_indent
            .as_ref()
            .is_none_or(|s| s.starts_with(&whitespace))
        {
            Some(whitespace)
        } else {
            cur_indent
        };
    }

    source
        .lines()
        .map(|l| l.replacen(cur_indent.as_deref().unwrap_or_default(), "", 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_backticks(source: &str) -> String {
    source.replace("```", "`\u{200B}``")
}

static GITHUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn github(captures: regex::Captures<'_>) -> Result<Vec<serenity::CreateComponent<'static>>> {
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

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```".to_owned()
                + language
                + "\n"
                + &truncate(&escape_backticks(&dedent(&selected_lines)), 2048)
                + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# GitHub · <t:{}:F>",
            chrono::Utc::now().timestamp()
        ))),
        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
    ])
}

static CODEBERG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://codeberg\.org/(?P<repo>[\w\-]+/[\w.\-]+)/src/(?P<ref_type>\S+?)/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn codeberg(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
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

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```".to_owned()
                + language
                + "\n"
                + &truncate(&escape_backticks(&dedent(&selected_lines)), 2048)
                + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# Codeburger · <t:{}:F>",
            chrono::Utc::now().timestamp()
        ))),
        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
    ])
}

static GITLAB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://gitlab\.com/(?P<repo>[\w\-]+/[\w.\-]+)/-/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument]
async fn gitlab(captures: regex::Captures<'_>) -> Result<Vec<serenity::CreateComponent<'static>>> {
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

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {repo} {file} L{start}{}",
            end.map(|end| format!("-{end}")).unwrap_or_default()
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```".to_owned()
                + language
                + "\n"
                + &truncate(&escape_backticks(&dedent(&selected_lines)), 2048)
                + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# GitLab · <t:{}:F>",
            chrono::Utc::now().timestamp()
        ))),
        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
    ])
}

static RUST_PLAYGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

#[tracing::instrument]
async fn rust_playground(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
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

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {gist_id}",
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```rust\n".to_owned() + &truncate(&escape_backticks(&dedent(&gist)), 2048) + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# play.rust-lang.org · <t:{}:F>",
            chrono::Utc::now().timestamp()
        ))),
        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
    ])
}

static GO_PLAYGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument]
async fn go_playground(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
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

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {id}",
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```go\n".to_owned() + &truncate(&escape_backticks(&dedent(&code)), 2048) + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# go.dev/play · <t:{}:F>",
            chrono::Utc::now().timestamp()
        ))),
        serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
    ])
}

#[expect(clippy::type_complexity)]
pub async fn resolve(content: &str) -> Result<Vec<serenity::CreateComponent<'static>>> {
    let mut components_tasks: Vec<
        Pin<
            Box<dyn Future<Output = Result<Vec<serenity::CreateComponent<'static>>>> + Send + Sync>,
        >,
    > = Vec::new();

    for captures in GITHUB.captures_iter(content) {
        components_tasks.push(Box::pin(async move { github(captures).await }));
    }

    for captures in CODEBERG.captures_iter(content) {
        components_tasks.push(Box::pin(async move { codeberg(captures).await }));
    }

    for captures in GITLAB.captures_iter(content) {
        components_tasks.push(Box::pin(async move { gitlab(captures).await }));
    }

    for captures in RUST_PLAYGROUND.captures_iter(content) {
        components_tasks.push(Box::pin(async move { rust_playground(captures).await }));
    }

    for captures in GO_PLAYGROUND.captures_iter(content) {
        components_tasks.push(Box::pin(async move { go_playground(captures).await }));
    }

    let mut components = futures_util::future::join_all(components_tasks)
        .await
        .into_iter()
        .filter_map(|r| match r {
            Ok(c) => Some(c),
            Err(err) => {
                warn!("{err:?}");
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    components.pop();

    Ok(components)
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

    let components = resolve(&message.content).await?;

    if !components.is_empty() {
        suppress_embeds(ctx, message).await?;

        message
            .channel_id
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .components(components)
                    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::default().replied_user(false),
                    )
                    .reference_message(message),
            )
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedent_works() {
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("\ta"), "a");
        assert_eq!(dedent("    a"), "a");
        assert_eq!(dedent("a\n\tb\nc"), "a\n\tb\nc");
        assert_eq!(dedent("\ta\n\t\tb\n\tc"), "a\n\tb\nc");
        assert_eq!(dedent("  a\n    b\n  c"), "a\n  b\nc");
        assert_eq!(dedent("a  \n  b  \nc  "), "a  \n  b  \nc  ");
        assert_eq!(dedent("  a  \n    b  \n  c  "), "a  \n  b  \nc  ");
    }
}
