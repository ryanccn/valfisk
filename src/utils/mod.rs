// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

pub mod axum;
pub mod error_handling;
pub mod serenity;

mod pluralize;
pub use pluralize::Pluralize;

pub fn truncate(s: &str, new_len: usize) -> String {
    let mut s = s.to_owned();
    s.truncate(new_len);
    s
}
