use axum::routing::{get, post};

use crate::{DbPool, auth, files, users};

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub jwt_secret: String,
    pub max_upload_size: usize,
}

pub fn build_state(
    pool: DbPool,
    s3_client: aws_sdk_s3::Client,
    s3_bucket: String,
    jwt_secret: String,
    max_upload_size: usize,
) -> AppState {
    AppState {
        pool,
        s3_client,
        s3_bucket,
        jwt_secret,
        max_upload_size,
    }
}

pub fn build_router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/users", post(users::handler::create_user))
        .route("/users/{id}", get(users::handler::get_user))
        .route("/auth/login", post(auth::handler::login))
        .route("/files", post(files::handler::upload_file))
        .route(
            "/files/{id}",
            get(files::handler::get_file).delete(files::handler::delete_file),
        )
        .route("/files/{id}/content", get(files::handler::download_file))
        .route(
            "/files/{id}/shares",
            post(files::handler::create_share_link),
        )
        .route("/s/{token}", get(files::handler::download_shared_file))
        .with_state(app_state)
}

pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result.expect("Failed to listen for Ctrl+C");
            }
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
    }
}
