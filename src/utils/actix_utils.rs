use actix_web::{body::BoxBody, http::StatusCode, HttpResponse};
use serde_json::json;

pub struct ActixError(color_eyre::eyre::Error);

impl std::fmt::Debug for ActixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ActixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl actix_web::ResponseError for ActixError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::InternalServerError()
            .json(json!({ "error": "An internal server error occurred!" }))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E> From<E> for ActixError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
