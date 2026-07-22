use dotenvy::dotenv;
use filers::{
    app::{self, shutdown_signal},
    db,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to read .env file");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&database_url);
    let upload_dir = std::env::var("UPLOAD_DIR").expect("UPLOAD_DIR must be set");

    let app_state = app::build_state(pool.clone(), upload_dir);
    let app = app::build_router(app_state);
    let server = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(server, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    drop(pool);
}
