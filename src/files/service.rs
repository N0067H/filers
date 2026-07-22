use axum::http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    files::{
        model::{File as StoredFile, FileShare, NewFile, NewFileShare},
        repo,
    },
};

pub async fn save_upload(
    state: &AppState,
    filename: String,
    display_name: String,
    data: Vec<u8>,
) -> Result<(), AppError> {
    fs::create_dir_all(&state.upload_dir)
        .await
        .map_err(crate::error::internal_error)?;

    let path = state.upload_dir.join(&filename);
    let mut file = File::create(&path)
        .await
        .map_err(crate::error::internal_error)?;

    file.write_all(&data)
        .await
        .map_err(crate::error::internal_error)?;

    let new_file = NewFile {
        name: filename,
        display_name,
        path: path.to_string_lossy().to_string(),
        size: data.len() as i64,
    };

    repo::insert_file(state.pool.clone(), new_file)
        .await
        .map_err(crate::error::internal_error)
}

pub async fn get_file(state: &AppState, id: i32) -> Result<StoredFile, AppError> {
    repo::find_file_by_id(state.pool.clone(), id)
        .await
        .map_err(crate::error::internal_error)
}

pub async fn delete_file(state: &AppState, id: i32) -> Result<StoredFile, AppError> {
    let file = get_file(state, id).await?;
    fs::remove_file(&file.path)
        .await
        .map_err(crate::error::internal_error)?;
    Ok(file)
}

pub async fn create_share_link(state: &AppState, id: i32) -> Result<FileShare, AppError> {
    let new_share = NewFileShare {
        file_id: id,
        token: Uuid::new_v4(),
    };

    repo::insert_share_link(state.pool.clone(), new_share)
        .await
        .map_err(crate::error::internal_error)
}

pub async fn get_shared_file(state: &AppState, token: Uuid) -> Result<StoredFile, AppError> {
    let share_link = repo::find_share_by_token(state.pool.clone(), token)
        .await
        .map_err(crate::error::internal_error)?;

    if let Some(expires_at) = share_link.expires_at {
        if expires_at < chrono::Utc::now().naive_utc() {
            return Err((StatusCode::FORBIDDEN, "Link has expired".to_string()));
        }
    }

    if share_link.revoked_at.is_some() {
        return Err((StatusCode::FORBIDDEN, "Link has been revoked".to_string()));
    }

    get_file(state, share_link.file_id).await
}

pub async fn download_content(file: &StoredFile) -> Result<(HeaderMap, Vec<u8>), AppError> {
    let bytes = tokio::fs::read(&file.path)
        .await
        .map_err(crate::error::internal_error)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    let content_disposition = format!(r#"attachment; filename="{}""#, file.name);
    let header_value = HeaderValue::from_str(&content_disposition)
        .map_err(crate::error::internal_error)?;
    headers.insert(CONTENT_DISPOSITION, header_value);

    Ok((headers, bytes))
}
