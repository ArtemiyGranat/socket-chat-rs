create table if not exists users (
  id bigserial primary key,
  username text unique not null
);