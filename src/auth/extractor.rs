use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{
    AppState,
    auth::service,
    errors::app_error::AppError,
    users::{repo, service::UserResponse},
};

pub struct AuthenticatedUser {
    pub user: UserResponse,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized("Missing authorization header"))?;

        let token = authorization
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized("Invalid authorization scheme"))?;

        let user_id = service::decode_token(&app_state, token)?;
        let user = repo::find_user_by_id(app_state.pool.clone(), user_id)
            .await
            .map_err(|_| AppError::Unauthorized("User not found"))?;

        Ok(Self { user: user.into() })
    }
}
