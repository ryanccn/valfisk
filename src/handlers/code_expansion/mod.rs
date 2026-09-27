// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;
use regex::Regex;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::{Result, bail};
use std::{collections::HashSet, sync::LazyLock};

use crate::{
    analytics,
    http::HTTP,
    storage::code_expansion::CodeExpansionData,
    utils::{serenity::suppress_embeds, sha256},
};

mod forges;
mod github;
mod playgrounds;
mod render;

pub struct Expansion {
    heading: Option<String>,
    accent: Option<u32>,
    parent: Option<String>,
    meta: Option<String>,
    body: Option<Body>,
    source: &'static str,
    link: String,
}

struct Body {
    text: String,
    language: Option<String>,
}

impl Expansion {
    fn snippet(
        heading: String,
        language: &str,
        code: &str,
        source: &'static str,
        link: &str,
    ) -> Self {
        Self {
            heading: Some(heading),
            accent: None,
            parent: None,
            meta: None,
            body: Some(Body {
                text: escape_backticks(code),
                language: Some(language.to_owned()),
            }),
            source,
            link: link.to_owned(),
        }
    }
}

fn dedent(source: &str) -> String {
    let indent_of = |line: &str| line.len() - line.trim_start().len();

    let indent = source
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| &line[..indent_of(line)])
        .reduce(|common, indent| {
            let shared = common
                .char_indices()
                .zip(indent.chars())
                .find(|((_, a), b)| a != b)
                .map_or_else(|| common.len().min(indent.len()), |((i, _), _)| i);

            &common[..shared]
        })
        .unwrap_or_default();

    source
        .lines()
        .map(|l| l.strip_prefix(indent).unwrap_or(l))
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

fn heading_with_range(prefix: &str, start: usize, end: Option<usize>) -> String {
    format!(
        "{prefix} L{start}{}",
        end.map(|end| format!("-{end}")).unwrap_or_default()
    )
}

fn plural(count: u64, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

async fn fetch_text(url: String) -> Result<String> {
    Ok(HTTP
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?)
}

async fn expand_file(
    raw_url: String,
    repo: &str,
    file: &str,
    (start, end): (usize, Option<usize>),
    source: &'static str,
    link: &str,
) -> Result<Expansion> {
    let body = select_lines(&fetch_text(raw_url).await?, start, end)?;

    Ok(Expansion::snippet(
        heading_with_range(&format!("{repo} {file}"), start, end),
        extension_of(file),
        &dedent(&body),
        source,
        link,
    ))
}

fn extension_of(file: &str) -> &str {
    file.split('.').next_back().unwrap_or_default()
}

macro_rules! sources {
    ($($source:ident => $regex:path, $handler:path;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Source {
            $($source,)*
        }

        // More specific patterns come first, since links matched by them are not matched again by later patterns.
        static SOURCES: &[(Source, &LazyLock<Regex>)] = &[$((Source::$source, &$regex),)*];

        #[tracing::instrument(skip(captures), fields(link = captures.get_match().as_str()))]
        async fn expand(source: Source, captures: regex::Captures<'_>) -> Result<Expansion> {
            tracing::debug!("expanding link");

            match source {
                $(Source::$source => $handler(captures).await,)*
            }
        }
    };
}

sources! {
    GitHubDiff => github::DIFF, github::diff;
    GitHubReviewComment => github::REVIEW_COMMENT, github::review_comment;
    GitHubReview => github::REVIEW, github::review;
    GitHubComment => github::COMMENT, github::comment;
    GitHubFile => github::FILE, github::file;
    GitHubCommit => github::COMMIT, github::commit;
    GitHubIssue => github::ISSUE, github::issue;
    Tangled => forges::TANGLED, forges::tangled;
    TangledStrings => forges::TANGLED_STRINGS, forges::tangled_strings;
    Codeberg => forges::CODEBERG, forges::codeberg;
    GitLab => forges::GITLAB, forges::gitlab;
    RustPlayground => playgrounds::RUST, playgrounds::rust;
    GoPlayground => playgrounds::GO, playgrounds::go;
}

const MAX_EXPANSIONS: usize = 8;

fn find(content: &str) -> (Vec<(Source, regex::Captures<'_>)>, usize) {
    let mut found: Vec<(Source, regex::Captures<'_>)> = Vec::new();

    for (source, regex) in SOURCES {
        for captures in regex.captures_iter(content) {
            let range = captures.get_match().range();

            if !found.iter().any(|(_, existing)| {
                let existing = existing.get_match().range();
                existing.start < range.end && range.start < existing.end
            }) {
                found.push((*source, captures));
            }
        }
    }

    found.sort_unstable_by_key(|(_, captures)| captures.get_match().start());

    let mut seen = HashSet::new();
    found.retain(|(_, captures)| seen.insert(captures.get_match().as_str()));

    let omitted = found.len().saturating_sub(MAX_EXPANSIONS);
    found.truncate(MAX_EXPANSIONS);

    (found, omitted)
}

pub async fn resolve(content: &str) -> Vec<serenity::CreateComponent<'static>> {
    let (found, omitted) = find(content);

    let expansions = futures_util::future::join_all(
        found
            .into_iter()
            .map(|(source, captures)| expand(source, captures)),
    )
    .await
    .into_iter()
    .filter_map(|result| result.inspect_err(|err| tracing::warn!("{err:?}")).ok())
    .collect();

    render::render(expansions, omitted)
}

fn ignored(ctx: &serenity::Context, message: &serenity::Message) -> bool {
    message.author.id == ctx.cache.current_user().id
        || message
            .flags
            .is_some_and(|f| f.contains(serenity::MessageFlags::SUPPRESS_NOTIFICATIONS))
}

#[tracing::instrument(skip_all, fields(message = message.id.get()))]
pub async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if ignored(ctx, message) {
        return Ok(());
    }

    let components = resolve(&message.content).await;

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
    if ignored(ctx, message) {
        return Ok(());
    }

    let content_hash = BASE64.encode(sha256(message.content.as_bytes()));

    if let Some(storage) = &ctx.data::<crate::Data>().storage
        && let Some(existing) = storage.get_code_expansion(message.id).await?
        && content_hash != existing.content_hash
    {
        let components = resolve(&message.content).await;

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
                        content_hash,
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
    fn dedent_leaves_mismatched_indentation() {
        assert_eq!(dedent("  a\n\t\tb"), "  a\n\t\tb");
        assert_eq!(dedent("\t a\n\t b"), "a\nb");
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

    #[test]
    fn find_prefers_specific_patterns() {
        let sources = |content: &str| {
            find(content)
                .0
                .into_iter()
                .map(|(source, _)| source)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            sources("https://github.com/a/b/pull/1#issuecomment-2"),
            [Source::GitHubComment]
        );
        assert_eq!(
            sources("https://github.com/a/b/pull/1/files#r2"),
            [Source::GitHubReviewComment]
        );
        assert_eq!(
            sources("https://github.com/a/b/pull/1#pullrequestreview-2"),
            [Source::GitHubReview]
        );
        assert_eq!(
            sources(&format!(
                "https://github.com/a/b/pull/1/files#diff-{}R2",
                "a".repeat(64)
            )),
            [Source::GitHubDiff]
        );
        assert_eq!(
            sources("https://github.com/a/b/pull/1/commits/abcdef1"),
            [Source::GitHubCommit]
        );
    }
}
