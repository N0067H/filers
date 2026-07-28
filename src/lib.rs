pub mod app;
pub mod auth;
pub mod db;
pub mod errors;
pub mod files;
pub mod storage;
pub mod users;

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub use app::AppState;
