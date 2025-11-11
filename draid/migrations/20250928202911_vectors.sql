-- Add migration script here
CREATE TABLE vectors (
    id bigserial PRIMARY KEY,
    document_id bigserial REFERENCES documents(id),
    kb_id bigserial REFERENCES knowledge_bases(id),
    content text NOT NULL,
    embedding vector(1024)
);
CREATE INDEX ON vectors USING hnsw (embedding vector_cosine_ops) WITH (m = 16, ef_construction = 64);
CREATE INDEX ON vectors (kb_id);
