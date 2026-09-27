// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use regex::Regex;
use reqwest::header;
use std::sync::LazyLock;

use super::{
    Expansion, dedent, expand_file, extension_of, heading_with_range, line_range, select_lines,
};
use crate::http::HTTP;

pub static TANGLED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://tangled\.org/(?P<repo>@[\w.\-]+/[\w.\-]+)/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn tangled(captures: regex::Captures<'_>) -> Result<Expansion> {
    let (repo, r#ref, file) = (&captures["repo"], &captures["ref"], &captures["file"]);
    let url = format!("https://tangled.org/{repo}/raw/{ref}/{file}");

    expand_file(
        url,
        repo,
        file,
        line_range(&captures)?,
        "Tangled",
        &captures[0],
    )
    .await
}

pub static TANGLED_STRINGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://tangled\.org/strings/(?P<string>@?[\w.\-]+/\w+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn tangled_strings(captures: regex::Captures<'_>) -> Result<Expansion> {
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

    Ok(Expansion::snippet(
        heading_with_range(string, start, end),
        &language,
        &dedent(&body),
        "Tangled Strings",
        &captures[0],
    ))
}

pub static CODEBERG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://codeberg\.org/(?P<repo>[\w\-]+/[\w.\-]+)/src/(?P<ref_type>\S+?)/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn codeberg(captures: regex::Captures<'_>) -> Result<Expansion> {
    let (repo, ref_type, r#ref, file) = (
        &captures["repo"],
        &captures["ref_type"],
        &captures["ref"],
        &captures["file"],
    );
    let url = format!("https://codeberg.org/{repo}/raw/{ref_type}/{ref}/{file}");

    expand_file(
        url,
        repo,
        file,
        line_range(&captures)?,
        "Codeberg",
        &captures[0],
    )
    .await
}

pub static GITLAB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://gitlab\.com/(?P<repo>[\w\-]+/[\w.\-]+)/-/blob/(?P<ref>\S+?)/(?P<file>[^\s?]+)(\?\S*)?#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn gitlab(captures: regex::Captures<'_>) -> Result<Expansion> {
    let (repo, r#ref, file) = (&captures["repo"], &captures["ref"], &captures["file"]);
    let url = format!("https://gitlab.com/{repo}/-/raw/{ref}/{file}");

    expand_file(
        url,
        repo,
        file,
        line_range(&captures)?,
        "GitLab",
        &captures[0],
    )
    .await
}
