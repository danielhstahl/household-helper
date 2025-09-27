use pgvector::Vector;
use rocket::serde::Serialize;
use sqlx::{Error, Pool, Postgres, Row, postgres::PgRow};
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SimilarContent {
    content: String,
}
pub async fn get_similar_content(
    embeddings: Vec<f32>,
    num_matches: i16,
    pool: &Pool<Postgres>,
) -> sqlx::Result<Vec<SimilarContent>> {
    let embeddings = Vector::from(embeddings);
    //cosine similarity
    let rows = sqlx::query(
        r#"
        SELECT content FROM vectors ORDER BY embedding <=> $1 LIMIT $2;
        "#,
    )
    .bind(embeddings)
    .bind(num_matches)
    .fetch_all(pool)
    .await?;
    let result: Result<Vec<_>, Error> = rows
        .iter()
        .map(|v: &PgRow| {
            let content = v.try_get("content")?;
            Ok(SimilarContent { content })
        })
        .collect();
    Ok(result?)
}
pub async fn write_content(
    content: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    pool: &Pool<Postgres>,
) -> sqlx::Result<()> {
    let mut query_string = String::from("INSERT INTO vectors (content, embedding) VALUES ");

    // Generate the multi-row `VALUES` placeholders
    // hilariously hacky...put the numbers in by dollar sign
    let embedding_placeholders: Vec<String> = (0..content.len())
        .map(|i| format!("(${}, ${})", 2 * i + 1, 2 * i + 2))
        .collect();

    query_string.push_str(&embedding_placeholders.join(", "));

    // Create a `Query` object with the dynamic SQL string
    let mut sqlx_query = sqlx::query(&query_string);

    // Bind each value individually to the query, including the enum
    for (text, embedding) in content.into_iter().zip(embeddings.into_iter()) {
        sqlx_query = sqlx_query.bind(text).bind(Vector::from(embedding));
    }
    sqlx_query.execute(pool).await?;
    Ok(())
}

pub async fn write_single_content(
    content: &str,
    embeddings: Vec<f32>,
    pool: &Pool<Postgres>,
) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO vectors (content, embedding) VALUES ($1, $2)")
        .bind(&content)
        .bind(Vector::from(embeddings))
        .execute(pool)
        .await?;

    Ok(())
}
