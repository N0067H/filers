create table users (
    id serial primary key,
    username varchar not null,
    email varchar not null unique,
    password_hash varchar not null
);

create table files (
    id serial primary key,
    owner_id integer not null references users(id) on delete cascade,
    name varchar not null,
    display_name varchar not null,
    storage_key varchar not null,
    size bigint not null,
    content_type varchar not null,
    created_at timestamp not null default now()
);

create table file_shares (
    id serial primary key,
    file_id integer not null references files(id) on delete cascade,
    token uuid not null unique,
    expires_at timestamp null,
    revoked_at timestamp null,
    created_at timestamp not null default now()
);
