use dotenvy::dotenv;
use filers::{
    app::{self, shutdown_signal},
    db,
    storage::s3,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to read .env file");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&database_url);
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let s3_bucket = std::env::var("S3_BUCKET").expect("S3_BUCKET must be set");
    let max_upload_size = std::env::var("MAX_UPLOAD_SIZE")
        .expect("MAX_UPLOAD_SIZE must be set")
        .parse::<usize>()
        .expect("MAX_UPLOAD_SIZE must be a valid usize");

    let s3_client = s3::create_s3_client().await;

    let app_state = app::build_state(
        pool.clone(),
        s3_client,
        s3_bucket,
        jwt_secret,
        max_upload_size,
    );
    let app = app::build_router(app_state);
    let server = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(server, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    drop(pool);
}
