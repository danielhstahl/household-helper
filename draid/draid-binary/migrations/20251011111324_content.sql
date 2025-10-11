-- Add migration script here
CREATE TABLE content (
    document_id bigserial PRIMARY KEY REFERENCES documents(id),
    content text NOT NULL
);
