-- Remember to create a user and assign to a new schema for better security
-- I hate cockroachdb and i'm going to migrate back to psql..

begin;

create table if not exists pastes (
    id bigserial primary key,
    paste text not null check(octet_length(paste) <= 1048576),
    created_at timestamptz default now(),
    name varchar(128) default 'New PasteIt',
    language varchar(32) default 'plaintext'
);

create index idx_pastes_create_at on pastes (created_at);

create table if not exists paste_rate_limits (
    encrypted_ip varchar(64) primary key not null, -- technically not encrypted, just hashed but eh..
    paste_count int4 not null default 1,
    last_reset timestamp with time zone not null default current_timestamp
);

create index idx_paste_rate_limits_last_reset on paste_rate_limits (last_reset);

end;