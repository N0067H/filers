use std::time::Duration;

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub async fn run_db<T, F>(pool: crate::DbPool, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut diesel::PgConnection) -> Result<T, diesel::result::Error> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|err| err.to_string())?;
        f(&mut conn).map_err(|err| err.to_string())
    })
    .await
    .map_err(|err| err.to_string())?
}

pub fn create_pool(database_url: &str) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(5)
        .idle_timeout(Some(Duration::from_secs(30)))
        .build(manager)
        .expect("Failed to create database pool")
}
