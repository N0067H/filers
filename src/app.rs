use axum::routing::{get, post};

use crate::{DbPool, files};

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub upload_dir: std::path::PathBuf,
}

pub fn build_state(pool: DbPool, upload_dir: String) -> AppState {
    AppState {
        pool,
        upload_dir: upload_dir.into(),
    }
}

pub fn build_router(app_state: AppState) -> axum::Router {
    axum::Router::new()
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
