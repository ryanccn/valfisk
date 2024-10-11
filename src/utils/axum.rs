// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

pub struct AxumError(eyre::Report);

impl IntoResponse for AxumError {
    fn into_response(self) -> Response {
        error!("{}", self.0);

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

pub type AxumResult<T> = Result<T, AxumError>;
