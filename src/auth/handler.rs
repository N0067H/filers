use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{AppState, auth::service, errors::app_error::AppError};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<service::LoginResponse>, AppError> {
    Ok(Json(service::login(&state, payload.email, payload.password).await?))
}
