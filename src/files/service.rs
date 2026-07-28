use axum::http::{
    HeaderMap, HeaderValue,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use uuid::Uuid;

use crate::{
    AppState,
    errors::{
        app_error::AppError,
        repo_error::{RepoError, map_repo_error},
    },
    files::{
        model::{File as StoredFile, FileShare, NewFile, NewFileShare},
        repo,
    },
};

pub async fn save_upload(
    state: &AppState,
    owner_id: i32,
    filename: String,
    display_name: String,
    storage_key: String,
    content_type: String,
    size: i64,
) -> Result<(), AppError> {
    let new_file = NewFile {
        owner_id,
        name: filename,
        display_name,
        storage_key,
        content_type,
        size,
    };

    repo::insert_file(state.pool.clone(), new_file)
        .await
        .map_err(map_repo_error)
}

pub async fn get_file(state: &AppState, id: i32) -> Result<StoredFile, AppError> {
    repo::find_file_by_id(state.pool.clone(), id)
        .await
        .map_err(|err| match err {
            RepoError::NotFound => AppError::NotFound("File not found"),
            other => map_repo_error(other),
        })
}

pub async fn get_owned_file(
    state: &AppState,
    owner_id: i32,
    id: i32,
) -> Result<StoredFile, AppError> {
    repo::find_file_by_id_and_owner(state.pool.clone(), id, owner_id)
        .await
        .map_err(|err| match err {
            RepoError::NotFound => AppError::Forbidden("You do not have access to this file"),
            other => map_repo_error(other),
        })
}

pub async fn delete_file(state: &AppState, owner_id: i32, id: i32) -> Result<StoredFile, AppError> {
    let file = get_owned_file(state, owner_id, id).await?;

    state
        .s3_client
        .delete_object()
        .bucket(&state.s3_bucket)
        .key(&file.storage_key)
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "s3 delete_object failed: bucket={}, key={}, error={error:?}",
                state.s3_bucket, file.storage_key
            );
            AppError::internal(error)
        })?;

    repo::delete_file_by_id_and_owner(state.pool.clone(), id, owner_id)
        .await
        .map_err(map_repo_error)
}

pub async fn create_share_link(
    state: &AppState,
    owner_id: i32,
    id: i32,
) -> Result<FileShare, AppError> {
    get_owned_file(state, owner_id, id).await?;

    let new_share = NewFileShare {
        file_id: id,
        token: Uuid::new_v4(),
    };

    repo::insert_share_link(state.pool.clone(), new_share)
        .await
        .map_err(map_repo_error)
}

pub async fn get_shared_file(state: &AppState, token: Uuid) -> Result<StoredFile, AppError> {
    let share_link = repo::find_share_by_token(state.pool.clone(), token)
        .await
        .map_err(|err| match err {
            RepoError::NotFound => AppError::NotFound("Share link not found"),
            other => map_repo_error(other),
        })?;

    if let Some(expires_at) = share_link.expires_at {
        if expires_at < chrono::Utc::now().naive_utc() {
            return Err(AppError::Forbidden("Link has expired"));
        }
    }

    if share_link.revoked_at.is_some() {
        return Err(AppError::Forbidden("Link has been revoked"));
    }

    get_file(state, share_link.file_id).await
}

pub async fn download_content(
    state: &AppState,
    file: &StoredFile,
) -> Result<(HeaderMap, Vec<u8>), AppError> {
    let bytes = state
        .s3_client
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&file.storage_key)
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "s3 get_object failed: bucket={}, key={}, error={error:?}",
                state.s3_bucket, file.storage_key
            );
            AppError::internal(error)
        })?
        .body
        .collect()
        .await
        .map_err(|error| {
            eprintln!(
                "s3 get_object body collect failed: bucket={}, key={}, error={error:?}",
                state.s3_bucket, file.storage_key
            );
            AppError::internal(error)
        })?
        .into_bytes();

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&file.content_type).map_err(AppError::internal)?,
    );
    let content_disposition = format!(r#"attachment; filename="{}""#, file.name);
    let header_value = HeaderValue::from_str(&content_disposition).map_err(AppError::internal)?;
    headers.insert(CONTENT_DISPOSITION, header_value);

    Ok((headers, bytes.to_vec()))
}
