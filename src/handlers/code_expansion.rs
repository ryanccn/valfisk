// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;
use regex::Regex;
use reqwest::header;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::{Result, bail};
use std::{pin::Pin, sync::LazyLock};

use crate::{
    analytics,
    http::HTTP,
    storage::code_expansion::CodeExpansionData,
    utils::{serenity::suppress_embeds, sha256, truncate},
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
    source.replace("```", "`\u{200D}``")
}

const MAX_LINES: usize = 256;

fn line_range(captures: &regex::Captures<'_>) -> Result<(usize, Option<usize>)> {
    let start = captures["start"].parse::<usize>()?;

    if start == 0 {
        bail!("line number must be at least 1");
    }

    let end = captures
        .name("end")
        .and_then(|end| end.as_str().parse::<usize>().ok());

    if let Some(end) = end
        && end < start
    {
        bail!("end line number precedes start line number");
    }

    Ok((start, end))
}

fn select_lines(source: &str, start: usize, end: Option<usize>) -> Result<String> {
    let count = end.unwrap_or(start) - start + 1;

    if count > MAX_LINES {
        bail!("requested line range exceeds {MAX_LINES} lines");
    }

    let selected = source
        .lines()
        .skip(start - 1)
        .take(count)
        .collect::<Vec<_>>();

    if selected.len() < count {
        bail!("out of bounds line indexes");
    }

    Ok(selected.join("\n"))
}

fn render_snippet(
    heading: &str,
    language: &str,
    body: &str,
    source_label: &str,
    link: &str,
) -> Vec<serenity::CreateComponent<'static>> {
    vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {heading}"
        ))),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
            "```".to_owned()
                + language
                + "\n"
                + truncate(&escape_backticks(&dedent(body)), 2048)
                + "\n```",
        )),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# [{source_label}]({link}) · {}",
            serenity::FormattedTimestamp::now()
        ))),
        serenity::CreateComponent::Separator(
            serenity::CreateSeparator::new()
                .divider(true)
                .spacing(serenity::SeparatorSpacingSize::Large),
        ),
    ]
}

fn heading_with_range(prefix: &str, start: usize, end: Option<usize>) -> String {
    format!(
        "{prefix} L{start}{}",
        end.map(|end| format!("-{end}")).unwrap_or_default()
    )
}

async fn expand_file(
    raw_url: String,
    prefix: &str,
    language: &str,
    source_label: &str,
    link: &str,
    start: usize,
    end: Option<usize>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
    let source = HTTP
        .get(raw_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let body = select_lines(&source, start, end)?;

    Ok(render_snippet(
        &heading_with_range(prefix, start, end),
        language,
        &body,
        source_label,
        link,
    ))
}

fn extension_of(file: &str) -> &str {
    file.split('.').next_back().unwrap_or_default()
}

static GITHUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all)]
async fn github(captures: regex::Captures<'_>) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling GitHub link");

    let repo = &captures["repo"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let (start, end) = line_range(&captures)?;

    expand_file(
        format!("https://raw.githubusercontent.com/{repo}/{ref}/{file}"),
        &format!("{repo} {file}"),
        extension_of(file),
        "GitHub",
        &captures[0],
        start,
        end,
    )
    .await
}

static GITHUB_COMMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/(?P<type>issues|pull)/(?P<issue>\d+)#issuecomment-(?P<comment>\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct GitHubComment {
    body: String,
    user: GitHubCommentUser,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct GitHubCommentUser {
    login: String,
    html_url: String,
}

#[tracing::instrument(skip_all)]
async fn github_comment(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling GitHub comment link");

    let repo = &captures["repo"];
    let issue = &captures["issue"];
    let comment = &captures["comment"];

    let comment: GitHubComment = HTTP
        .get(format!(
            "https://api.github.com/repos/{repo}/issues/comments/{comment}"
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(vec![
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### {repo} #{issue}",
        ))),
        serenity::CreateComponent::Container(serenity::CreateContainer::new(vec![
            serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                format!(
                    "-# [@{}]({}) · {}",
                    comment.user.login,
                    comment.user.html_url,
                    serenity::FormattedTimestamp::new(comment.created_at.into(), None)
                ),
            )),
            serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
                comment.body,
            )),
        ])),
        serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "-# [GitHub]({}) · {}",
            &captures[0],
            serenity::FormattedTimestamp::now()
        ))),
        serenity::CreateComponent::Separator(
            serenity::CreateSeparator::new()
                .divider(true)
                .spacing(serenity::SeparatorSpacingSize::Large),
        ),
    ])
}

static TANGLED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://tangled\.org/(?P<repo>@[\w.\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all)]
async fn tangled(captures: regex::Captures<'_>) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling Tangled link");

    let repo = &captures["repo"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let (start, end) = line_range(&captures)?;

    expand_file(
        format!("https://tangled.org/{repo}/raw/{ref}/{file}"),
        &format!("{repo} {file}"),
        extension_of(file),
        "Tangled",
        &captures[0],
        start,
        end,
    )
    .await
}

static TANGLED_STRINGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://tangled\.org/strings/(?P<string>@?[\w.\-]+/\w+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all)]
async fn tangled_strings(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling Tangled strings link");

    let string = &captures["string"];

    let (start, end) = line_range(&captures)?;

    let resp = HTTP
        .get(format!("https://tangled.org/strings/{string}/raw"))
        .send()
        .await?
        .error_for_status()?;

    let language = resp
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("inline; filename=\""))
        .and_then(|s| s.strip_suffix("\""))
        .map(|s| extension_of(s).to_owned())
        .unwrap_or_default();

    let source = resp.text().await?;
    let body = select_lines(&source, start, end)?;

    Ok(render_snippet(
        &heading_with_range(string, start, end),
        &language,
        &body,
        "Tangled Strings",
        &captures[0],
    ))
}

static CODEBERG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://codeberg\.org/(?P<repo>[\w\-]+/[\w.\-]+)/src/(?P<ref_type>\S+?)/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all)]
async fn codeberg(
    captures: regex::Captures<'_>,
) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling Codeberg link");

    let repo = &captures["repo"];
    let ref_type = &captures["ref_type"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let (start, end) = line_range(&captures)?;

    expand_file(
        format!("https://codeberg.org/{repo}/raw/{ref_type}/{ref}/{file}"),
        &format!("{repo} {file}"),
        extension_of(file),
        "Codeberg",
        &captures[0],
        start,
        end,
    )
    .await
}

static GITLAB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://gitlab\.com/(?P<repo>[\w\-]+/[\w.\-]+)/-/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all)]
async fn gitlab(captures: regex::Captures<'_>) -> Result<Vec<serenity::CreateComponent<'static>>> {
    tracing::debug!(link = &captures[0], "handling GitLab link");

    let repo = &captures["repo"];
    let r#ref = &captures["ref"];
    let file = &captures["file"];

    let (start, end) = line_range(&captures)?;

    expand_file(
        format!("https://gitlab.com/{repo}/-/raw/{ref}/{file}"),
        &format!("{repo} {file}"),
        extension_of(file),
        "GitLab",
        &captures[0],
        start,
        end,
    )
    .await
}

static RUST_PLAYGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

#[tracing::instrument(skip_all)]
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

    Ok(render_snippet(
        gist_id,
        "rust",
        &gist,
        "play.rust-lang.org",
        &captures[0],
    ))
}

static GO_PLAYGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument(skip_all)]
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

    Ok(render_snippet(id, "go", &code, "go.dev/play", &captures[0]))
}

#[expect(clippy::type_complexity)]
pub async fn resolve(content: &str) -> Result<Vec<serenity::CreateComponent<'static>>> {
    let mut components_tasks: Vec<
        Pin<
            Box<
                dyn Future<Output = (usize, Result<Vec<serenity::CreateComponent<'static>>>)>
                    + Send
                    + Sync,
            >,
        >,
    > = Vec::new();

    macro_rules! collect_expansions {
        ($($regex:ident => $handler:ident),+ $(,)?) => {
            $(
                for captures in $regex.captures_iter(content) {
                    let start = captures.get_match().start();
                    components_tasks
                        .push(Box::pin(async move { (start, $handler(captures).await) }));
                }
            )+
        };
    }

    collect_expansions!(
        GITHUB => github,
        GITHUB_COMMENT => github_comment,
        TANGLED => tangled,
        TANGLED_STRINGS => tangled_strings,
        CODEBERG => codeberg,
        GITLAB => gitlab,
        RUST_PLAYGROUND => rust_playground,
        GO_PLAYGROUND => go_playground,
    );

    let mut results = futures_util::future::join_all(components_tasks)
        .await
        .into_iter()
        .filter_map(|(start, result)| match result {
            Ok(result) => Some((start, result)),
            Err(err) => {
                tracing::warn!("{err:?}");
                None
            }
        })
        .collect::<Vec<_>>();
    results.sort_unstable_by_key(|v| v.0);

    let mut components = results.into_iter().flat_map(|v| v.1).collect::<Vec<_>>();
    components.pop();

    Ok(components)
}

#[tracing::instrument(skip_all, fields(message = message.id.get()))]
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
        let _ = suppress_embeds(ctx, message).await;

        let new_message = message
            .channel_id
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::default().replied_user(false),
                    )
                    .components(components)
                    .reference_message(message),
            )
            .await?;

        if let Some(storage) = &ctx.data::<crate::Data>().storage {
            storage
                .set_code_expansion(
                    message.id,
                    CodeExpansionData {
                        message: new_message.id,
                        content_hash: BASE64.encode(sha256(message.content.as_bytes())),
                    },
                )
                .await?;
        }

        analytics::send_event("code_expansion_v1", message.guild_id);
    }

    Ok(())
}

#[tracing::instrument(skip_all, fields(message = message.id.get()))]
pub async fn handle_edit(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if message
        .flags
        .is_some_and(|f| f.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
    {
        return Ok(());
    }

    if let Some(storage) = &ctx.data::<crate::Data>().storage
        && let Some(existing) = storage.get_code_expansion(message.id).await?
        && sha256(message.content.as_bytes()) != BASE64.decode(&existing.content_hash)?
    {
        let components = resolve(&message.content).await?;

        if components.is_empty() {
            message
                .channel_id
                .delete_message(&ctx.http, existing.message, None)
                .await?;

            storage.del_code_expansion(message.id).await?;
        } else {
            message
                .channel_id
                .edit_message(
                    &ctx.http,
                    existing.message,
                    serenity::EditMessage::default()
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                        .allowed_mentions(
                            serenity::CreateAllowedMentions::default().replied_user(false),
                        )
                        .components(components),
                )
                .await?;

            storage
                .set_code_expansion(
                    message.id,
                    CodeExpansionData {
                        message: existing.message,
                        content_hash: BASE64.encode(sha256(message.content.as_bytes())),
                    },
                )
                .await?;

            analytics::send_event("code_expansion_v1", message.guild_id);
        }
    }

    Ok(())
}

#[tracing::instrument(skip(ctx))]
pub async fn handle_delete(
    ctx: &serenity::Context,
    channel: serenity::GenericChannelId,
    message: serenity::MessageId,
) -> Result<()> {
    if let Some(storage) = &ctx.data::<crate::Data>().storage
        && let Some(existing) = storage.get_code_expansion(message).await?
    {
        channel
            .delete_message(&ctx.http, existing.message, None)
            .await?;
        storage.del_code_expansion(message).await?;
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

    #[test]
    fn select_lines_works() {
        let source = "a\nb\nc\nd";

        assert_eq!(select_lines(source, 1, None).unwrap(), "a");
        assert_eq!(select_lines(source, 2, Some(3)).unwrap(), "b\nc");
        assert_eq!(select_lines(source, 1, Some(4)).unwrap(), "a\nb\nc\nd");
        assert_eq!(select_lines(source, 4, Some(4)).unwrap(), "d");

        assert!(select_lines(source, 5, None).is_err());
        assert!(select_lines(source, 3, Some(9)).is_err());
    }

    #[test]
    fn select_lines_rejects_oversized_ranges() {
        let source = "a\n".repeat(MAX_LINES * 4);

        assert!(select_lines(&source, 1, Some(MAX_LINES)).is_ok());
        assert!(select_lines(&source, 1, Some(MAX_LINES + 1)).is_err());
        assert!(select_lines(&source, 1, Some(usize::MAX)).is_err());
    }
}
