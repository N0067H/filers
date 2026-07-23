use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::{
    AppState,
    errors::app_error::AppError,
    files::{model, service},
};

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    fs::create_dir_all(&state.upload_dir)
        .await
        .map_err(AppError::internal)?;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(AppError::internal)?
    {
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let display_name = field
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| filename.clone());

        let stored_filename = Uuid::new_v4().to_string();
        let path = state.upload_dir.join(&stored_filename);
        let mut file = File::create(&path)
            .await
            .map_err(AppError::internal)?;
        let mut size = 0usize;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(AppError::internal)?
        {
            size += chunk.len();

            if size > state.max_upload_size {
                return Err(AppError::PayloadTooLarge("File too large"));
            }

            file.write_all(&chunk)
                .await
                .map_err(AppError::internal)?;
        }

        service::save_upload(
            &state,
            filename,
            display_name,
            path.to_string_lossy().to_string(),
            size as i64,
        )
        .await?;
    }

    Ok(StatusCode::CREATED)
}

pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::File>, AppError> {
    Ok(Json(service::get_file(&state, id).await?))
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let file = service::get_file(&state, id).await?;
    service::download_content(&file).await
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::File>, AppError> {
    Ok(Json(service::delete_file(&state, id).await?))
}

pub async fn create_share_link(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::FileShare>, AppError> {
    Ok(Json(service::create_share_link(&state, id).await?))
}

pub async fn download_shared_file(
    State(state): State<AppState>,
    Path(token): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let file = service::get_shared_file(&state, token).await?;
    service::download_content(&file).await
}
