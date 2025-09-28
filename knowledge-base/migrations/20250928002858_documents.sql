-- Add migration script here
CREATE TABLE documents (id bigserial PRIMARY KEY, hash char(64) NOT NULL);
CREATE UNIQUE INDEX ON documents (hash);
