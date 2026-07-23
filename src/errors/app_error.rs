use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum AppError {
    BadRequest(&'static str),
    NotFound(&'static str),
    Forbidden(&'static str),
    PayloadTooLarge(&'static str),
    Internal(String),
}

impl AppError {
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        Self::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.to_string()),
            AppError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}
