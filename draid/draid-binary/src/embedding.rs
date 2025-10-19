use crate::psql_vectors::{write_chunk_content, write_document};
use async_openai::{
    Client, config::OpenAIConfig, error::OpenAIError, types::CreateEmbeddingRequestArgs,
};
use futures::{StreamExt, stream};
use sha256::digest;
use sqlx::PgPool;
use text_splitter::TextSplitter;
use tracing::info;
use uuid::Uuid;
#[derive(Clone)]
pub struct EmbeddingClient {
    llm: Client<OpenAIConfig>,
    model_name: String,
}
impl EmbeddingClient {
    pub fn new(model_name: String, api_endpoint: &str) -> Self {
        Self {
            model_name,
            llm: get_embedding_client(api_endpoint),
        }
    }
}
fn get_embedding_client(api_endpoint: &str) -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::default().with_api_base(format!("{}/v1", api_endpoint)))
}

pub async fn get_embeddings(
    client: &EmbeddingClient,
    message: &str,
) -> Result<Vec<f32>, OpenAIError> {
    let request = CreateEmbeddingRequestArgs::default()
        .model(&client.model_name)
        .input([message])
        .build()?;
    let response = client.llm.embeddings().create(request).await?;
    Ok(response.data.into_iter().next().unwrap().embedding)
}

async fn extract_and_write(
    client: &EmbeddingClient,
    document_id: i64,
    kb_id: i64,
    chunk: String,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let embeddings = get_embeddings(&client, &chunk).await?;
    write_chunk_content(document_id, kb_id, &chunk, embeddings, pool).await?;
    Ok(())
}
pub async fn ingest_content(
    kb_id: i64, //category of knowledge base
    pool: &PgPool,
    raw_file: Vec<u8>,
    client: &EmbeddingClient,
) -> Result<String, anyhow::Error> {
    let max_characters = 1000;
    let splitter = TextSplitter::new(max_characters);
    let span_id = Uuid::new_v4().to_string();
    info!(
        tool_use = true,
        is_kb = true,
        span_id,
        "Started ingesting content"
    );
    let content = String::from_utf8(raw_file)?;
    let content_hash = digest(&content);
    match write_document(&content_hash, &content, pool).await {
        Ok(document_id) => {
            info!(
                tool_use = true,
                endpoint = "ingest",
                span_id,
                "Finished reading content"
            );
            let chunks: Vec<String> = splitter.chunks(&content).map(|v| v.to_string()).collect();
            info!(
                tool_use = true,
                endpoint = "ingest",
                span_id,
                "Finished chunking content"
            );
            info!(
                tool_use = true,
                endpoint = "ingest",
                span_id,
                message = format!("Number of chunks {}", chunks.len())
            );
            let futures = chunks.into_iter().map(|chunk| async move {
                extract_and_write(&client, document_id, kb_id, chunk, pool).await
            });
            let results: Vec<anyhow::Result<()>> = stream::iter(futures)
                .buffer_unordered(100) // Concurrently process up to 100 tasks
                .collect()
                .await;
            info!(
                tool_use = true,
                endpoint = "ingest",
                span_id,
                "Finished writing vectors"
            );
            for result in results {
                result?;
            }
            Ok(content_hash)
        }
        Err(_e) => {
            info!(
                tool_use = true,
                endpoint = "ingest",
                span_id,
                "Already indexed!"
            );
            Ok(content_hash)
        }
    }
}
