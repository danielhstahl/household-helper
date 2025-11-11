-- Add migration script here
CREATE TABLE knowledge_bases (id bigserial PRIMARY KEY, name varchar(64) NOT NULL);
CREATE UNIQUE INDEX ON knowledge_bases (name);
