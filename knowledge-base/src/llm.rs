use ollama_rs::Ollama;
use ollama_rs::error::OllamaError;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;

#[derive(Clone)]
pub struct EmbeddingClient {
    ollama: Ollama,
    model: String,
}
pub fn get_embedding_client(api_endpoint: &str, model: String) -> EmbeddingClient {
    let parsed_url: Vec<&str> = api_endpoint.split(":").collect();
    let url = format!("{}:{}", parsed_url[0], parsed_url[1]);
    let port: u16 = parsed_url[2].parse().unwrap();
    EmbeddingClient {
        ollama: Ollama::new(&url, port),
        model,
    }
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

pub async fn get_embeddings_batch(
    client: &EmbeddingClient,
    messages: Vec<String>,
) -> Result<Vec<Vec<f32>>, OllamaError> {
    let request = GenerateEmbeddingsRequest::new(client.model.clone(), messages.into());
    Ok(client.ollama.generate_embeddings(request).await?.embeddings)
}
//curl --header "Content-Type: application/json"  -X POST http://127.0.0.1:8000/content/similar --data '{"text": "hello world!", "num_results": 3}'
//curl -X POST http://127.0.0.1:8000/content/ingest --data '@paul_graham_essay.txt'
