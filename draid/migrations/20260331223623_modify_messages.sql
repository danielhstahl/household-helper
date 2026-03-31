-- Add migration script here
ALTER TABLE messages add column reasoning text not null DEFAULT '';
