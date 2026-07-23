pub mod app;
pub mod db;
pub mod errors;
pub mod files;

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub use app::AppState;
