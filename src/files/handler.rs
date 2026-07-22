use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, internal_error},
    files::{model, service},
};

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let display_name = field
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| filename.clone());

        let data = field.bytes().await.map_err(internal_error)?.to_vec();
        service::save_upload(&state, filename, display_name, data).await?;
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
