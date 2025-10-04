-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id UUID NOT NULL PRIMARY KEY,
    username varchar(255) NOT NULL,
    hashed_password varchar(255) NOT NULL
);
