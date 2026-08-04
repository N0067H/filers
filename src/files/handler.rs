use aws_sdk_s3::{
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
};
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State, multipart::Field},
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    AppState,
    auth::extractor::AuthenticatedUser,
    errors::app_error::AppError,
    files::{
        model,
        service::{self, FileResponse},
    },
};

const MULTIPART_PART_SIZE: usize = 5 * 1024 * 1024;

pub async fn upload_file(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<FileResponse>), AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::BadRequest("No file provided"))?;

    let filename = field
        .file_name()
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let display_name = filename.clone();

    let content_type = field
        .content_type()
        .map(str::to_owned)
        .unwrap_or_else(|| "application/octet-stream".to_owned());

    let storage_key = format!("files/{}", Uuid::new_v4());

    let size = upload_field_to_s3(&state, &storage_key, &content_type, field).await?;

    let file = match service::save_upload(
        &state,
        user.user.id,
        filename,
        display_name,
        storage_key.clone(),
        content_type,
        size as i64,
    )
    .await
    {
        Ok(file) => file,
        Err(error) => {
            if let Err(delete_error) = state
                .s3_client
                .delete_object()
                .bucket(&state.s3_bucket)
                .key(&storage_key)
                .send()
                .await
            {
                eprintln!(
                    "failed to delete S3 object after DB save failure: key={}, error={}",
                    storage_key, delete_error
                );
            }

            return Err(error);
        }
    };

    Ok((StatusCode::CREATED, Json(file)))
}

async fn upload_field_to_s3(
    state: &AppState,
    storage_key: &str,
    content_type: &str,
    mut field: Field<'_>,
) -> Result<usize, AppError> {
    let mut total_size = 0usize;
    let mut buffer = Vec::new();
    let mut upload_id: Option<String> = None;
    let mut completed_parts = Vec::new();
    let mut part_number = 1i32;

    while let Some(chunk) = field.next().await {
        let chunk: Bytes = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                if let Some(upload_id) = upload_id.as_deref() {
                    abort_multipart_upload(state, storage_key, upload_id).await;
                }

                return Err(AppError::internal(error));
            }
        };

        total_size = total_size
            .checked_add(chunk.len())
            .ok_or(AppError::PayloadTooLarge("File too large"))?;

        if total_size > state.max_upload_size {
            if let Some(upload_id) = upload_id.as_deref() {
                abort_multipart_upload(state, storage_key, upload_id).await;
            }
            return Err(AppError::PayloadTooLarge("File too large"));
        }

        buffer.extend_from_slice(&chunk);

        if upload_id.is_none() && buffer.len() > MULTIPART_PART_SIZE {
            upload_id = Some(create_multipart_upload(state, storage_key, content_type).await?);
        }

        while upload_id.is_some() && buffer.len() >= MULTIPART_PART_SIZE {
            let part = buffer.drain(..MULTIPART_PART_SIZE).collect::<Vec<_>>();
            let completed_part = match upload_part(
                state,
                storage_key,
                upload_id.as_deref().expect("upload id is set"),
                part_number,
                part,
            )
            .await
            {
                Ok(part) => part,
                Err(error) => {
                    abort_multipart_upload(
                        state,
                        storage_key,
                        upload_id.as_deref().expect("upload id is set"),
                    )
                    .await;
                    return Err(error);
                }
            };

            completed_parts.push(completed_part);
            part_number += 1;
        }
    }

    match upload_id {
        None => {
            state
                .s3_client
                .put_object()
                .bucket(&state.s3_bucket)
                .key(storage_key)
                .content_type(content_type)
                .content_length(buffer.len() as i64)
                .body(ByteStream::from(buffer))
                .send()
                .await
                .map_err(|error| {
                    eprintln!(
                        "s3 put_object failed: bucket={}, key={}, error={error:?}",
                        state.s3_bucket, storage_key
                    );
                    AppError::internal(error)
                })?;
        }
        Some(upload_id) => {
            let completed_part =
                match upload_part(state, storage_key, &upload_id, part_number, buffer).await {
                    Ok(part) => part,
                    Err(error) => {
                        abort_multipart_upload(state, storage_key, &upload_id).await;
                        return Err(error);
                    }
                };

            completed_parts.push(completed_part);

            let multipart_upload = CompletedMultipartUpload::builder()
                .set_parts(Some(completed_parts))
                .build();

            if let Err(error) = state
                .s3_client
                .complete_multipart_upload()
                .bucket(&state.s3_bucket)
                .key(storage_key)
                .upload_id(&upload_id)
                .multipart_upload(multipart_upload)
                .send()
                .await
            {
                eprintln!(
                    "s3 complete_multipart_upload failed: bucket={}, key={}, upload_id={}, error={error:?}",
                    state.s3_bucket, storage_key, upload_id
                );
                abort_multipart_upload(state, storage_key, &upload_id).await;
                return Err(AppError::internal(error));
            }
        }
    }

    Ok(total_size)
}

async fn create_multipart_upload(
    state: &AppState,
    storage_key: &str,
    content_type: &str,
) -> Result<String, AppError> {
    let output = state
        .s3_client
        .create_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(storage_key)
        .content_type(content_type)
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "s3 create_multipart_upload failed: bucket={}, key={}, error={error:?}",
                state.s3_bucket, storage_key
            );
            AppError::internal(error)
        })?;

    output
        .upload_id()
        .map(str::to_owned)
        .ok_or_else(|| AppError::internal("missing upload_id from create_multipart_upload"))
}

async fn upload_part(
    state: &AppState,
    storage_key: &str,
    upload_id: &str,
    part_number: i32,
    part: Vec<u8>,
) -> Result<CompletedPart, AppError> {
    let content_length = part.len() as i64;
    let output = state
        .s3_client
        .upload_part()
        .bucket(&state.s3_bucket)
        .key(storage_key)
        .upload_id(upload_id)
        .part_number(part_number)
        .content_length(content_length)
        .body(ByteStream::from(part))
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "s3 upload_part failed: bucket={}, key={}, upload_id={}, part_number={}, error={error:?}",
                state.s3_bucket, storage_key, upload_id, part_number
            );
            AppError::internal(error)
        })?;

    Ok(CompletedPart::builder()
        .part_number(part_number)
        .set_e_tag(output.e_tag().map(str::to_owned))
        .build())
}

async fn abort_multipart_upload(state: &AppState, storage_key: &str, upload_id: &str) {
    let _ = state
        .s3_client
        .abort_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(storage_key)
        .upload_id(upload_id)
        .send()
        .await;
}

pub async fn get_file(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::File>, AppError> {
    Ok(Json(
        service::get_owned_file(&state, user.user.id, id).await?,
    ))
}

pub async fn download_file(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let file = service::get_owned_file(&state, user.user.id, id).await?;
    service::download_content(&state, &file).await
}

pub async fn delete_file(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::File>, AppError> {
    Ok(Json(service::delete_file(&state, user.user.id, id).await?))
}

pub async fn create_share_link(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::FileShare>, AppError> {
    Ok(Json(
        service::create_share_link(&state, user.user.id, id).await?,
    ))
}

pub async fn download_shared_file(
    State(state): State<AppState>,
    Path(token): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let file = service::get_shared_file(&state, token).await?;
    service::download_content(&state, &file).await
}
