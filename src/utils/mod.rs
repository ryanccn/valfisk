// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::ops::{Deref, DerefMut};
use tokio::task;

pub mod serenity;

mod axum;
pub use axum::AxumResult;

mod error_handling;
pub use error_handling::ValfiskError;

mod exit_code_error;
pub use exit_code_error::ExitCodeError;

mod pluralize;
pub use pluralize::Pluralize;

pub fn truncate(s: &str, new_len: usize) -> String {
    s.chars().take(new_len).collect()
}

pub struct JoinHandleAbortOnDrop<T>(task::JoinHandle<T>);

impl<T> Deref for JoinHandleAbortOnDrop<T> {
    type Target = task::JoinHandle<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for JoinHandleAbortOnDrop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Drop for JoinHandleAbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub fn spawn_abort_on_drop<F>(future: F) -> JoinHandleAbortOnDrop<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandleAbortOnDrop(task::spawn(future))
}
