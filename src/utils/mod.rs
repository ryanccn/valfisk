// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use tokio::task::JoinHandle;

pub mod axum;
pub mod error_handling;
pub mod serenity;

mod pluralize;
pub use pluralize::Pluralize;

pub fn truncate(s: &str, new_len: usize) -> String {
    s.chars().take(new_len).collect()
}

pub struct JoinHandleAbortOnDrop<T>(JoinHandle<T>);

impl<T> std::ops::Deref for JoinHandleAbortOnDrop<T> {
    type Target = JoinHandle<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for JoinHandleAbortOnDrop<T> {
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
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandleAbortOnDrop(tokio::task::spawn(future))
}
