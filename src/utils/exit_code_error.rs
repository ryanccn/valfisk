// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::process::ExitCode;

#[derive(Debug)]
pub struct ExitCodeError(pub u8);

impl ExitCodeError {
    pub fn as_std(&self) -> ExitCode {
        ExitCode::from(self.0)
    }
}

impl std::fmt::Display for ExitCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ExitCodeError {}
