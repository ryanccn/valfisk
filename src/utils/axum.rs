// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub struct AxumError(eyre::Report);

impl IntoResponse for AxumError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "An internal server error occurred!" })),
        )
            .into_response()
    }
}

impl<E> From<E> for AxumError
where
    E: Into<eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[expect(dead_code)]
pub type AxumResult<T> = Result<T, AxumError>;
