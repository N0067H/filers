use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    errors::{
        app_error::AppError,
        repo_error::{RepoError, map_repo_error},
    },
    users::{repo, service::UserResponse},
};

const TOKEN_TTL_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: i32,
    exp: usize,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: UserResponse,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(AppError::internal)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    verify(password, password_hash).map_err(AppError::internal)
}

pub async fn login(
    state: &AppState,
    email: String,
    password: String,
) -> Result<LoginResponse, AppError> {
    let user = repo::find_user_by_email(state.pool.clone(), email)
        .await
        .map_err(|err| match err {
            RepoError::NotFound => AppError::Unauthorized("Invalid email or password"),
            other => map_repo_error(other),
        })?;

    let password_ok = verify_password(&password, &user.password_hash)?;
    if !password_ok {
        return Err(AppError::Unauthorized("Invalid email or password"));
    }

    let access_token = generate_token(state, user.id)?;

    Ok(LoginResponse {
        access_token,
        user: user.into(),
    })
}

pub fn generate_token(state: &AppState, user_id: i32) -> Result<String, AppError> {
    let expiration = Utc::now() + Duration::hours(TOKEN_TTL_HOURS);
    let claims = Claims {
        sub: user_id,
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(AppError::internal)
}

pub fn decode_token(state: &AppState, token: &str) -> Result<i32, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token"))?;

    Ok(token_data.claims.sub)
}
