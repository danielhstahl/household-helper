-- Add migration script here
CREATE TABLE IF NOT EXISTS sessions
(
    id UUID NOT NULL PRIMARY KEY,
    username_id UUID NOT NULL references users(id),
    session_start TIMESTAMPTZ NOT NULL
);

CREATE INDEX sessions_user_index ON sessions(username_id);
