// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::sync::LazyLock;

pub static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap()
});
