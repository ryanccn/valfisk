// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use once_cell::sync::Lazy;

pub static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    reqwest::ClientBuilder::new()
        .user_agent(user_agent)
        .build()
        .unwrap()
});
