// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use chrono::{DateTime, Utc};
use eyre::{Result, bail, eyre};
use regex::Regex;
use reqwest::header;
use serde::de::DeserializeOwned;
use std::{
    fmt::{self, Write as _},
    sync::LazyLock,
};

use super::{Body, Expansion, MAX_LINES, escape_backticks, expand_file, line_range, plural};
use crate::{config::CONFIG, http::HTTP, utils::sha256};

const OPEN: u32 = 0x3fb950;
const DONE: u32 = 0xa371f7;
const CLOSED: u32 = 0xf85149;
const MUTED: u32 = 0x9198a1;

async fn api<T: DeserializeOwned>(path: &str) -> Result<T> {
    let mut request = HTTP
        .get(format!("https://api.github.com{path}"))
        .header(header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10");

    if let Some(token) = &CONFIG.github_token {
        request = request.bearer_auth(token);
    }

    Ok(request.send().await?.error_for_status()?.json().await?)
}

#[derive(serde::Deserialize, Debug, Clone)]
struct User {
    login: String,
    html_url: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[@{}]({})", self.login, self.html_url)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
struct Label {
    name: String,
}

struct Summary {
    reference: String,
    title: String,
    accent: Option<u32>,
    meta: String,
    labels: Vec<Label>,
}

impl Summary {
    async fn fetch(repo: &str, number: &str, pull: bool) -> Result<Self> {
        let (title, accent, meta, labels) = if pull {
            let pull: PullRequest = api(&format!("/repos/{repo}/pulls/{number}")).await?;

            let (state, accent) = if pull.merged {
                ("Merged", DONE)
            } else if pull.state == "closed" {
                ("Closed", CLOSED)
            } else if pull.draft {
                ("Draft", MUTED)
            } else {
                ("Open", OPEN)
            };

            let meta = format!(
                "-# {state} · {} · opened <t:{}:R> · {}\n-# +{} −{} · {} · `{}` ← `{}`",
                pull.user,
                pull.created_at.timestamp(),
                plural(pull.comments, "comment"),
                pull.additions,
                pull.deletions,
                plural(pull.changed_files, "file"),
                pull.base.r#ref,
                pull.head.label,
            );

            (pull.title, accent, meta, pull.labels)
        } else {
            let issue: Issue = api(&format!("/repos/{repo}/issues/{number}")).await?;

            let (state, accent) = match (
                issue.state.as_str(),
                issue.state_reason.as_deref(),
                &issue.pull_request,
            ) {
                (_, _, Some(IssuePullRequest { merged_at: Some(_) })) => ("Merged", DONE),
                ("open", ..) => ("Open", OPEN),
                (_, Some("not_planned"), _) => ("Closed as not planned", MUTED),
                (_, Some("duplicate"), _) => ("Closed as duplicate", MUTED),
                (_, _, Some(_)) => ("Closed", CLOSED),
                _ => ("Closed", DONE),
            };

            let meta = format!(
                "-# {state} · {} · opened <t:{}:R> · {}",
                issue.user,
                issue.created_at.timestamp(),
                plural(issue.comments, "comment"),
            );

            (issue.title, accent, meta, issue.labels)
        };

        Ok(Self {
            reference: format!("{repo} #{number}"),
            title: title.trim().to_owned(),
            accent: Some(accent),
            meta,
            labels,
        })
    }

    fn from_commit(repo: &str, commit: &Commit) -> Self {
        let title = commit.data.message.lines().next().unwrap_or_default();

        let mut meta = format!(
            "-# {} · <t:{}:R>",
            commit
                .author
                .as_ref()
                .map_or_else(|| commit.data.author.name.clone(), ToString::to_string),
            commit.data.author.date.timestamp(),
        );

        if let Some(stats) = &commit.stats {
            let _ = write!(meta, " · +{} −{}", stats.additions, stats.deletions);
        }

        let _ = write!(meta, " · {}", plural(commit.files.len() as u64, "file"));

        Self {
            reference: format!("{repo} {}", &commit.sha[..7]),
            title: title.trim().to_owned(),
            accent: None,
            meta,
            labels: Vec::new(),
        }
    }

    fn header(&self) -> String {
        let mut header = format!("-# {}\n### {}\n{}", self.reference, self.title, self.meta);

        if !self.labels.is_empty() {
            let _ = write!(
                header,
                "\n-# {}",
                self.labels
                    .iter()
                    .map(|label| format!("`{}`", label.name))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }

        header
    }

    fn card(self, meta: String, body: &str, link: &str) -> Expansion {
        let expansion = self.into_expansion(link);

        Expansion {
            parent: expansion.meta,
            meta: Some(meta),
            body: Some(Body {
                text: body.trim().to_owned(),
                language: None,
            }),
            ..expansion
        }
    }

    fn into_expansion(self, link: &str) -> Expansion {
        Expansion {
            heading: None,
            accent: self.accent,
            parent: None,
            meta: Some(self.header()),
            body: None,
            source: "GitHub",
            link: link.to_owned(),
        }
    }
}

pub static FILE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn file(captures: regex::Captures<'_>) -> Result<Expansion> {
    let (repo, r#ref, file) = (&captures["repo"], &captures["ref"], &captures["file"]);
    let url = format!("https://raw.githubusercontent.com/{repo}/{ref}/{file}");

    expand_file(
        url,
        repo,
        file,
        line_range(&captures)?,
        "GitHub",
        &captures[0],
    )
    .await
}

pub static COMMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/(?P<type>issues|pull)/(?P<issue>\d+)#issuecomment-(?P<comment>\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct Comment {
    body: String,
    user: User,
    created_at: DateTime<Utc>,
}

pub async fn comment(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let issue = &captures["issue"];
    let comment = &captures["comment"];

    let path = format!("/repos/{repo}/issues/comments/{comment}");
    let (comment, parent) = tokio::try_join!(
        api::<Comment>(&path),
        Summary::fetch(repo, issue, &captures["type"] == "pull")
    )?;

    Ok(parent.card(
        format!(
            "-# {} · <t:{}:R>",
            comment.user,
            comment.created_at.timestamp()
        ),
        &comment.body,
        &captures[0],
    ))
}

pub static REVIEW_COMMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/pull/(?P<pull>\d+)\S*?#(?:discussion_)?r(?P<comment>\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct ReviewComment {
    body: String,
    user: User,
    created_at: DateTime<Utc>,
    path: String,
    diff_hunk: String,
    original_line: Option<usize>,
    original_start_line: Option<usize>,
}

pub async fn review_comment(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let pull = &captures["pull"];
    let comment = &captures["comment"];

    let path = format!("/repos/{repo}/pulls/comments/{comment}");
    let (comment, parent) = tokio::try_join!(
        api::<ReviewComment>(&path),
        Summary::fetch(repo, pull, true)
    )?;

    let span = comment
        .original_start_line
        .zip(comment.original_line)
        .map_or(1, |(start, end)| end.saturating_sub(start) + 1)
        .clamp(4, 12);

    let hunk = comment
        .diff_hunk
        .lines()
        .filter(|line| !line.starts_with("@@"))
        .collect::<Vec<_>>();
    let hunk = hunk[hunk.len().saturating_sub(span)..].join("\n");

    let mut meta = format!(
        "-# {} · <t:{}:R> · `{}`",
        comment.user,
        comment.created_at.timestamp(),
        comment.path
    );

    if !hunk.is_empty() {
        let _ = write!(meta, "\n```diff\n{}\n```", escape_backticks(&hunk));
    }

    Ok(parent.card(meta, &comment.body, &captures[0]))
}

pub static REVIEW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/pull/(?P<pull>\d+)\S*?#pullrequestreview-(?P<review>\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct Review {
    body: String,
    user: User,
    state: String,
    submitted_at: Option<DateTime<Utc>>,
}

pub async fn review(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let pull = &captures["pull"];
    let review = &captures["review"];

    let path = format!("/repos/{repo}/pulls/{pull}/reviews/{review}");
    let (review, parent) =
        tokio::try_join!(api::<Review>(&path), Summary::fetch(repo, pull, true))?;

    let state = match review.state.as_str() {
        "APPROVED" => "Approved",
        "CHANGES_REQUESTED" => "Changes requested",
        "DISMISSED" => "Dismissed",
        _ => "Reviewed",
    };

    let mut meta = format!("-# {state} · {}", review.user);

    if let Some(submitted_at) = review.submitted_at {
        let _ = write!(meta, " · <t:{}:R>", submitted_at.timestamp());
    }

    Ok(parent.card(meta, &review.body, &captures[0]))
}

pub static ISSUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/(?P<type>issues|pull)/(?P<number>\d+)",
    )
    .unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct Issue {
    title: String,
    state: String,
    state_reason: Option<String>,
    user: User,
    labels: Vec<Label>,
    comments: u64,
    created_at: DateTime<Utc>,
    pull_request: Option<IssuePullRequest>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct IssuePullRequest {
    merged_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct PullRequest {
    title: String,
    state: String,
    draft: bool,
    merged: bool,
    user: User,
    labels: Vec<Label>,
    comments: u64,
    created_at: DateTime<Utc>,
    additions: u64,
    deletions: u64,
    changed_files: u64,
    base: Branch,
    head: Branch,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct Branch {
    #[serde(rename = "ref")]
    r#ref: String,
    label: String,
}

pub async fn issue(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let number = &captures["number"];

    Ok(Summary::fetch(repo, number, &captures["type"] == "pull")
        .await?
        .into_expansion(&captures[0]))
}

pub static COMMIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/(?:pull/\d+/)?commits?/(?P<sha>[0-9a-f]{7,40})\b").unwrap()
});

#[derive(serde::Deserialize, Debug, Clone)]
struct Commit {
    sha: String,
    #[serde(rename = "commit")]
    data: CommitData,
    author: Option<User>,
    stats: Option<CommitStats>,
    #[serde(default)]
    files: Vec<File>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct CommitData {
    message: String,
    author: CommitAuthor,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct CommitAuthor {
    name: String,
    date: DateTime<Utc>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct CommitStats {
    additions: u64,
    deletions: u64,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct File {
    filename: String,
    patch: Option<String>,
}

pub async fn commit(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let sha = &captures["sha"];

    let commit: Commit = api(&format!("/repos/{repo}/commits/{sha}")).await?;

    Ok(Summary::from_commit(repo, &commit).into_expansion(&captures[0]))
}

pub static DIFF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w\-]+/[\w.\-]+)/(?:pull/(?P<pull>\d+)/(?:files|changes)|commit/(?P<commit>[0-9a-f]{7,40}))#diff-(?P<hash>[0-9a-f]{64})(?:(?P<start>[LR]\d+)(?:-(?P<end>[LR]\d+))?)?").unwrap()
});

static HUNK_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^@@ -(\d+)(?:,\d+)? \+(\d+)").unwrap());

fn select_diff(patch: &str, start: &str, end: &str) -> Result<String> {
    let mut old = 0;
    let mut new = 0;
    let mut lines = Vec::new();

    for line in patch.lines() {
        if let Some(header) = HUNK_HEADER.captures(line) {
            old = header[1].parse::<usize>()?.saturating_sub(1);
            new = header[2].parse::<usize>()?.saturating_sub(1);
            lines.push((line, None, None));
            continue;
        }

        let (left, right) = match line.as_bytes().first() {
            Some(b'-') => {
                old += 1;
                (Some(old), None)
            }
            Some(b'+') => {
                new += 1;
                (None, Some(new))
            }
            Some(b'\\') => (None, None),
            _ => {
                old += 1;
                new += 1;
                (Some(old), Some(new))
            }
        };

        lines.push((line, left, right));
    }

    let position = |from: usize, point: &str| -> Result<usize> {
        let number = point[1..].parse::<usize>()?;

        lines[from..]
            .iter()
            .position(|&(_, left, right)| {
                (if point.starts_with('L') { left } else { right }) == Some(number)
            })
            .map(|index| index + from)
            .ok_or_else(|| eyre!("line {point} not found in diff"))
    };

    let start = position(0, start)?;
    let end = position(start, end)?;

    if end - start + 1 > MAX_LINES {
        bail!("requested line range exceeds {MAX_LINES} lines");
    }

    Ok(lines[start..=end]
        .iter()
        .map(|(line, ..)| *line)
        .collect::<Vec<_>>()
        .join("\n"))
}

pub async fn diff(captures: regex::Captures<'_>) -> Result<Expansion> {
    let repo = &captures["repo"];
    let hash = &captures["hash"];

    let is_target = |file: &File| hex::encode(sha256(file.filename.as_bytes())) == hash;

    let (parent, file) = if let Some(pull) = captures.name("pull") {
        let pull = pull.as_str();

        let (file, parent) = tokio::try_join!(
            async {
                for page in 1..=10 {
                    let files: Vec<File> = api(&format!(
                        "/repos/{repo}/pulls/{pull}/files?per_page=100&page={page}"
                    ))
                    .await?;

                    let last = files.len() < 100;
                    let file = files.into_iter().find(is_target);

                    if file.is_some() || last {
                        return Ok(file);
                    }
                }

                eyre::Ok(None)
            },
            Summary::fetch(repo, pull, true),
        )?;

        (parent, file)
    } else {
        let sha = &captures["commit"];
        let commit: Commit = api(&format!("/repos/{repo}/commits/{sha}")).await?;

        (
            Summary::from_commit(repo, &commit),
            commit.files.into_iter().find(is_target),
        )
    };

    let Some(file) = file else {
        bail!("could not find file in diff");
    };

    let Some(patch) = &file.patch else {
        bail!("diff for {} is unavailable", file.filename);
    };

    let mut meta = format!("**`{}`", file.filename);

    let patch = if let Some(start) = captures.name("start") {
        let start = start.as_str();
        let end = captures.name("end").map_or(start, |end| end.as_str());

        let _ = write!(meta, " · {start}");
        if end != start {
            let _ = write!(meta, "-{end}");
        }

        select_diff(patch, start, end)?
    } else {
        patch.to_owned()
    };

    meta.push_str("**");

    Ok(Expansion {
        body: Some(Body {
            text: escape_backticks(&patch),
            language: Some("diff".to_owned()),
        }),
        ..parent.card(meta, "", &captures[0])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATCH: &str = "@@ -1,4 +1,5 @@\n a\n-b\n+c\n+d\n e\n@@ -10,2 +11,2 @@\n x\n-y\n+z";

    #[test]
    fn select_diff_works() {
        assert_eq!(select_diff(PATCH, "R1", "R1").unwrap(), " a");
        assert_eq!(select_diff(PATCH, "L2", "L2").unwrap(), "-b");
        assert_eq!(select_diff(PATCH, "R2", "R3").unwrap(), "+c\n+d");
        assert_eq!(select_diff(PATCH, "L2", "R4").unwrap(), "-b\n+c\n+d\n e");
        assert_eq!(select_diff(PATCH, "L3", "L3").unwrap(), " e");
        assert_eq!(
            select_diff(PATCH, "R4", "R12").unwrap(),
            " e\n@@ -10,2 +11,2 @@\n x\n-y\n+z"
        );
        assert_eq!(select_diff(PATCH, "L11", "L11").unwrap(), "-y");
    }
}
