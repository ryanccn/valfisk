use actix_web::{body::BoxBody, http::StatusCode, HttpResponse};
use serde_json::json;

#[derive(Debug)]
pub struct ActixError(anyhow::Error);

impl std::fmt::Display for ActixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl actix_web::ResponseError for ActixError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::InternalServerError()
            .json(json!({"error": "An internal server error occurred!"}))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E> From<E> for ActixError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
