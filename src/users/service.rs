use serde::Serialize;

use crate::{
    AppState,
    errors::{
        app_error::AppError,
        repo_error::{RepoError, map_repo_error},
    },
    users::{
        model::{NewUser, User},
        repo,
    },
};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

pub async fn create_user(state: &AppState, new_user: NewUser) -> Result<UserResponse, AppError> {
    repo::insert_user(state.pool.clone(), new_user)
        .await
        .map(UserResponse::from)
        .map_err(map_repo_error)
}

pub async fn get_user(state: &AppState, user_id: i32) -> Result<UserResponse, AppError> {
    repo::find_user_by_id(state.pool.clone(), user_id)
        .await
        .map(UserResponse::from)
        .map_err(|err| match err {
            RepoError::NotFound => AppError::NotFound("User not found"),
            other => map_repo_error(other),
        })
}
