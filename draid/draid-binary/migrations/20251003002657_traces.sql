-- Add migration script here
CREATE TABLE IF NOT EXISTS traces
(
    span_id UUID NOT NULL,
    tool_use boolean NOT NULL,
    message text NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON traces (span_id);
