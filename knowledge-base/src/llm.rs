use ollama_rs::Ollama;
use ollama_rs::error::OllamaError;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use url::{ParseError, Url};
#[derive(Clone)]
pub struct EmbeddingClient {
    ollama: Ollama,
    model: String,
}
pub fn get_embedding_client(api_endpoint: &str, model: String) -> anyhow::Result<EmbeddingClient> {
    let url = Url::parse(api_endpoint)?;
    let port = url.port().ok_or_else(|| ParseError::InvalidPort)?;
    Ok(EmbeddingClient {
        ollama: Ollama::new(url, port),
        model,
    })
}

pub async fn get_embeddings(
    client: &EmbeddingClient,
    message: &str,
) -> Result<Vec<f32>, OllamaError> {
    let request = GenerateEmbeddingsRequest::new(client.model.clone(), message.into());
    Ok(client
        .ollama
        .generate_embeddings(request)
        .await?
        .embeddings
        .pop() //guaranteed to be one vector if single message
        .unwrap())
}
/*
pub async fn get_embeddings_batch(
    client: &EmbeddingClient,
    messages: Vec<String>,
) -> Result<Vec<Vec<f32>>, OllamaError> {
    let request = GenerateEmbeddingsRequest::new(client.model.clone(), messages.into());
    Ok(client.ollama.generate_embeddings(request).await?.embeddings)
}*/
