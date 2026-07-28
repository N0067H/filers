// @generated automatically by Diesel CLI.

diesel::table! {
    file_shares (id) {
        id -> Int4,
        file_id -> Int4,
        token -> Uuid,
        expires_at -> Nullable<Timestamp>,
        revoked_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Int4,
        owner_id -> Int4,
        name -> Varchar,
        display_name -> Varchar,
        storage_key -> Varchar,
        size -> Int8,
        content_type -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
    }
}

diesel::joinable!(file_shares -> files (file_id));
diesel::joinable!(files -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(file_shares, files, users,);
