// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use regex::Regex;
use std::sync::LazyLock;

use super::{Expansion, dedent, fetch_text};

pub static RUST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap()
});

pub async fn rust(captures: regex::Captures<'_>) -> Result<Expansion> {
    let gist = &captures["gist"];
    let code = fetch_text(format!(
        "https://gist.githubusercontent.com/rust-play/{gist}/raw/playground.rs"
    ))
    .await?;

    Ok(Expansion::snippet(
        gist.to_owned(),
        "rust",
        &dedent(&code),
        "play.rust-lang.org",
        &captures[0],
    ))
}

pub static GO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

pub async fn go(captures: regex::Captures<'_>) -> Result<Expansion> {
    let id = &captures["id"];
    let code = fetch_text(format!("https://go.dev/_/share?id={id}")).await?;

    Ok(Expansion::snippet(
        id.to_owned(),
        "go",
        &dedent(&code),
        "go.dev/play",
        &captures[0],
    ))
}
