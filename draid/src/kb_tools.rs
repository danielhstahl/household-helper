use crate::config::KB;
use crate::tools::Tool;
use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use std::sync::Arc;
#[derive(Clone)]
pub struct KBTool {
    name: String,
    description: String,
    url: String,
    num_results: i32,
}

impl KBTool {
    pub fn new(url: String, kb_config: KB) -> Self {
        let description = format!(
            "Knowledge base containing information on {}",
            kb_config.name
        );
        Self {
            name: kb_config.name,
            description,
            url,
            num_results: kb_config.num_results,
        }
    }
}

// Generated impl block, replacing placeholders with parsed values
#[async_trait::async_trait]
impl Tool for KBTool {
    fn name(&self) -> &String {
        // Use the generated static string literal
        &self.name
    }
    fn description(&self) -> &String {
        &self.description
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Search term to send to knowledge base",
                },
            },
            "required": ["content"],
        })
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let client = HttpClient::new();

        let kb_url = format!("{}/knowledge_base/{}/similar", self.url, self.name);

        let args: Value = serde_json::from_str(&args)?;
        let content = args["content"].as_str().unwrap();

        // The LitInt for num_results is directly interpolated
        let body = json!({"text": content, "num_results": self.num_results});

        let response = client.post(kb_url).json(&body).send().await?;
        let result = response.json::<Vec<String>>().await?;

        Ok(json!({"result": result}))
    }
}

//url cloning is unfortunate, but only happens at initial startup
pub fn get_tools(kb_configs: Vec<KB>, url: &str) -> Vec<Arc<dyn Tool + Send + Sync>> {
    kb_configs
        .into_iter()
        .map(|kb_config| {
            Arc::new(KBTool::new(url.to_string(), kb_config)) as Arc<dyn Tool + Send + Sync>
        })
        .collect()
}
