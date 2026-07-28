use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::files::schema::{file_shares, files};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = files)]
pub struct File {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
    pub display_name: String,
    pub storage_key: String,
    pub size: i64,
    pub content_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub owner_id: i32,
    pub name: String,
    pub display_name: String,
    pub storage_key: String,
    pub size: i64,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = file_shares)]
pub struct FileShare {
    pub id: i32,
    pub file_id: i32,
    pub token: uuid::Uuid,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = file_shares)]
pub struct NewFileShare {
    pub file_id: i32,
    pub token: uuid::Uuid,
}
