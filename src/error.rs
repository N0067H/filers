use axum::http::StatusCode;

pub type AppError = (StatusCode, String);

pub fn internal_error<E: std::fmt::Display>(err: E) -> AppError {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
