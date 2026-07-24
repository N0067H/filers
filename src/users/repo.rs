use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{
    DbPool,
    db::run_db,
    errors::repo_error::RepoError,
    users::{
        model::{NewUser, User},
        schema::{self, users},
    },
};

pub async fn insert_user(pool: DbPool, new_user: NewUser) -> Result<User, RepoError> {
    run_db(pool, move |conn| {
        diesel::insert_into(schema::users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    })
    .await
}

pub async fn find_user_by_id(pool: DbPool, user_id: i32) -> Result<User, RepoError> {
    run_db(pool, move |conn| {
        users::table
            .find(user_id)
            .select(User::as_select())
            .get_result(conn)
    })
    .await
}

pub async fn find_user_by_email(pool: DbPool, user_email: String) -> Result<User, RepoError> {
    run_db(pool, move |conn| {
        users::table
            .filter(schema::users::email.eq(user_email))
            .select(User::as_select())
            .get_result(conn)
    })
    .await
}
