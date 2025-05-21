// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use bytesize::ByteSize;

mod axum;
// pub use axum::AxumResult;
mod error_handling;
pub use error_handling::ValfiskError;
mod exit_code_error;
pub use exit_code_error::ExitCodeError;
mod nanoid;
pub use nanoid::nanoid;
pub mod serenity;

pub fn truncate(s: &str, new_len: usize) -> String {
    s.chars().take(new_len).collect()
}

pub fn format_bytes(bytes: u64) -> String {
    ByteSize::b(bytes).display().si().to_string()
}
