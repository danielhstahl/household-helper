use pgvector::Vector;
use poem_openapi::Object;
use serde::Serialize;
use sqlx::{Error, PgPool, Row, postgres::PgRow};

//if top num_matches are all in same document, will only return one document
pub async fn get_docs_with_similar_content(
    kb: i64,
    embeddings: Vec<f32>,
    num_matches: i16,
    pool: &PgPool,
) -> sqlx::Result<Vec<String>> {
    let embeddings = Vector::from(embeddings);
    //cosine similarity
    let rows = sqlx::query(
        r#"
        SELECT content FROM
        (
            SELECT DISTINCT document_id FROM (
                SELECT document_id FROM vectors
                WHERE kb_id=$1 ORDER BY embedding <=> $2 LIMIT $3
            )
        ) t1 INNER JOIN content t2 on t1.document_id=t2.document_id;
        "#,
    )
    .bind(kb)
    .bind(embeddings)
    .bind(num_matches)
    .fetch_all(pool)
    .await?;
    let result: Result<Vec<_>, Error> = rows
        .iter()
        .map(|v: &PgRow| {
            let content = v.try_get("content")?;
            Ok(content)
        })
        .collect();
    Ok(result?)
}

pub async fn write_chunk_content(
    document_id: i64,
    kb_id: i64,
    content: &str, //only for debugging, not exposed to anything
    embeddings: Vec<f32>,
    pool: &PgPool,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO vectors (document_id, kb_id, content, embedding) VALUES ($1, $2, $3, $4)",
    )
    .bind(&document_id)
    .bind(&kb_id)
    .bind(&content)
    .bind(Vector::from(embeddings))
    .execute(pool)
    .await?;

    Ok(())
}

struct IdOnly {
    id: i64,
}

//will error on index constraint
pub async fn write_document(
    document_hash: &str,
    document_content: &str,
    pool: &PgPool,
) -> sqlx::Result<i64> {
    let result = sqlx::query_as!(
        IdOnly,
        r#"INSERT INTO documents (hash) VALUES ($1) RETURNING id"#,
        document_hash
    )
    .fetch_one(pool)
    .await?;
    sqlx::query!(
        r#"INSERT INTO content (document_id, content) VALUES ($1, $2)"#,
        result.id,
        document_content,
    )
    .execute(pool)
    .await?;
    Ok(result.id)
}

pub async fn write_knowledge_base(name: &str, pool: &PgPool) -> sqlx::Result<i64> {
    let result = sqlx::query_as!(
        IdOnly,
        r#"INSERT INTO knowledge_bases (name) VALUES ($1) RETURNING id"#,
        name,
    )
    .fetch_one(pool)
    .await?;
    Ok(result.id)
}
#[derive(Debug, Serialize, Object)]
pub struct KnowledgeBase {
    pub id: i64,
    name: String,
}
pub async fn get_knowledge_bases(pool: &PgPool) -> sqlx::Result<Vec<KnowledgeBase>> {
    let result = sqlx::query_as!(KnowledgeBase, r#"SELECT id, name from knowledge_bases"#)
        .fetch_all(pool)
        .await?;
    Ok(result)
}

pub async fn get_knowledge_base(name: &str, pool: &PgPool) -> sqlx::Result<KnowledgeBase> {
    let result = sqlx::query_as!(
        KnowledgeBase,
        r#"SELECT id, name from knowledge_bases where name=$1"#,
        name
    )
    .fetch_one(pool)
    .await?;
    Ok(result)
}
