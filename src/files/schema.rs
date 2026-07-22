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
        name -> Varchar,
        display_name -> Varchar,
        path -> Varchar,
        size -> Int8,
        created_at -> Timestamp,
    }
}

diesel::joinable!(file_shares -> files (file_id));

diesel::allow_tables_to_appear_in_same_query!(file_shares, files);
