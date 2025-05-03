// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

pub mod serenity;

mod axum;
// pub use axum::AxumResult;

mod error_handling;
pub use error_handling::ValfiskError;

mod exit_code_error;
pub use exit_code_error::ExitCodeError;

mod pluralize;
// pub use pluralize::Pluralize;

pub fn truncate(s: &str, new_len: usize) -> String {
    s.chars().take(new_len).collect()
}

pub fn format_bytes(bytes: u64) -> String {
    bytesize::ByteSize::b(bytes).display().iec().to_string()
}
