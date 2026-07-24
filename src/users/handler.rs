use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    AppState,
    auth::service as auth_service,
    errors::app_error::AppError,
    users::{
        model::NewUser,
        service::{self, UserResponse},
    },
};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let password_hash = auth_service::hash_password(&payload.password)?;
    let new_user = NewUser {
        username: payload.username,
        email: payload.email,
        password_hash,
    };

    let user = service::create_user(&state, new_user).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(service::get_user(&state, id).await?))
}
