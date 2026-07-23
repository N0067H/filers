use crate::errors::app_error::AppError;

pub enum RepoError {
    NotFound,
    Db(String),
    Pool(String),
    Task(String),
}

pub fn map_repo_error(err: RepoError) -> AppError {
    match err {
        RepoError::NotFound => AppError::NotFound("Resource not found"),
        RepoError::Db(msg) | RepoError::Pool(msg) | RepoError::Task(msg) => {
            AppError::Internal(msg)
        }
    }
}
