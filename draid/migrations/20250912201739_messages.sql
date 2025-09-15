-- Add migration script here
DO $$ BEGIN
    CREATE TYPE message_type AS ENUM ('system', 'ai', 'human', 'tool');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
CREATE TABLE IF NOT EXISTS messages
(
    id UUID NOT NULL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id),
    message_type message_type NOT NULL,
    content text NOT NULL,
    message_ts TIMESTAMPTZ NOT NULL
);

CREATE INDEX messages_session_index ON messages(session_id);
