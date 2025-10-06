use async_openai::{
    Client, config::OpenAIConfig, error::OpenAIError, types::CreateEmbeddingRequestArgs,
};
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

/*
pub fn get_embedding_client(api_endpoint: &str, model: String) -> anyhow::Result<EmbeddingClient> {
    let url = Url::parse(api_endpoint)?;
    let port = url.port().ok_or_else(|| ParseError::InvalidPort)?;
    Ok(EmbeddingClient {
        ollama: Ollama::new(url, port),
        model,
    })
}
*/
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

/*
pub async fn get_embeddings_batch(
    client: &EmbeddingClient,
    messages: Vec<String>,
) -> Result<Vec<Vec<f32>>, OpenAIError> {
    let request = CreateEmbeddingRequestArgs::default()
        .model(&client.model_name)
        .input(messages)
        .build()?;
    let response = client.llm.embeddings().create(request).await?;
    println!("data size {}", response.data.len());
    Ok(response.data.into_iter().map(|v| v.embedding).collect())
}*/
/*
pub async fn get_embeddings_batch(
    client: &EmbeddingClient,
    messages: Vec<String>,
) -> Result<Vec<Vec<f32>>, OllamaError> {
    let request = GenerateEmbeddingsRequest::new(client.model.clone(), messages.into());
    Ok(client.ollama.generate_embeddings(request).await?.embeddings)
}*/
