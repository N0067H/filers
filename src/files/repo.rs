use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use crate::{
    DbPool,
    db::run_db,
    errors::repo_error::RepoError,
    files::{
        model::{File, FileShare, NewFile, NewFileShare},
        schema::{file_shares, files},
    },
};

pub async fn insert_file(pool: DbPool, new_file: NewFile) -> Result<(), RepoError> {
    run_db(pool, move |conn| {
        diesel::insert_into(files::table)
            .values(&new_file)
            .execute(conn)
            .map(|_| ())
    })
    .await
}

pub async fn find_file_by_id(pool: DbPool, id: i32) -> Result<File, RepoError> {
    run_db(pool, move |conn| {
        files::table
            .find(id)
            .select(File::as_select())
            .first(conn)
    })
    .await
}

pub async fn find_file_by_id_and_owner(
    pool: DbPool,
    id: i32,
    owner_id: i32,
) -> Result<File, RepoError> {
    run_db(pool, move |conn| {
        files::table
            .filter(files::id.eq(id))
            .filter(files::owner_id.eq(owner_id))
            .select(File::as_select())
            .first(conn)
    })
    .await
}

pub async fn delete_file_by_id_and_owner(
    pool: DbPool,
    id: i32,
    owner_id: i32,
) -> Result<File, RepoError> {
    run_db(pool, move |conn| {
        diesel::delete(
            files::table
                .filter(files::id.eq(id))
                .filter(files::owner_id.eq(owner_id)),
        )
        .returning(File::as_returning())
        .get_result(conn)
    })
    .await
}

pub async fn insert_share_link(pool: DbPool, new_share: NewFileShare) -> Result<FileShare, RepoError> {
    run_db(pool, move |conn| {
        diesel::insert_into(file_shares::table)
            .values(&new_share)
            .returning(FileShare::as_returning())
            .get_result(conn)
    })
    .await
}

pub async fn find_share_by_token(pool: DbPool, token: Uuid) -> Result<FileShare, RepoError> {
    run_db(pool, move |conn| {
        file_shares::table
            .filter(file_shares::token.eq(token))
            .select(FileShare::as_select())
            .first(conn)
    })
    .await
}
