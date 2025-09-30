use reqwest::Client as HttpClient;
use rocket::{
    serde,
    serde::json::{Value, json},
    serde::{Deserialize, Serialize},
};
use std::{ops::Deref, sync::Arc};
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Value;
    async fn invoke(&self, args: String) -> anyhow::Result<Value>;
}

#[async_trait::async_trait]
impl<P> Tool for P
where
    P: Deref<Target = dyn Tool + Send + Sync> + Send + Sync,
    // P must also satisfy other bounds you had on T, like Clone if needed
    // The inner dyn Tool must also satisfy Send + Sync for Arc to be safe
{
    fn name(&self) -> &'static str {
        self.deref().name()
    }
    fn description(&self) -> &'static str {
        self.deref().description()
    }
    fn parameters(&self) -> Value {
        self.deref().parameters()
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        self.deref().invoke(args).await
    }
}

#[derive(Debug)]
pub struct ToolError {
    pub name: String,
}

impl std::error::Error for ToolError {}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Tool {} not found", self.name)
    }
}

pub struct ToolRegistry {
    pub map: std::collections::HashMap<&'static str, Arc<dyn Tool + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }
    pub fn register<T: Tool + 'static + Send + Sync>(&mut self, t: T) {
        self.map.insert(t.name(), Arc::new(t));
    }
}

#[derive(Clone)]
pub struct AddTool;

#[async_trait::async_trait]
impl Tool for AddTool {
    fn name(&self) -> &'static str {
        "calculator"
    }
    fn description(&self) -> &'static str {
        "Add some numbers"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "a": {
                    "type": "number",
                    "description": "First number",
                },
                "b": {
                    "type": "number",
                    "description": "Second number",
                },
            },
            "required": ["a", "b"],
        })
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let args: Value = serde::json::from_str(&args)?;
        let result = args["a"].as_number().unwrap().as_f64().unwrap()
            + args["b"].as_number().unwrap().as_f64().unwrap();
        Ok(json!({"result":result}))
    }
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Content {
    content: String,
}

/*
#[derive(Clone)]
pub struct KnowledgeBase;

#[async_trait::async_trait]
impl Tool for KnowledgeBase {
    fn name(&self) -> &'static str {
        "knowledge_base"
    }
    fn description(&self) -> &'static str {
        "Knowledge base containing information on Paul Graham"
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
        let kb_url = "http://127.0.0.1:8001/content/similar";
        let args: Value = serde::json::from_str(&args)?;
        let content = args["content"].as_str().unwrap();
        let body = json!({"text":content, "num_results":3});
        let response = client.post(kb_url).json(&body).send().await?;
        let result = response.json::<Vec<Content>>().await?;
        println!("response from kb: {:?}", result);
        Ok(json!(result))
    }
}
*/

#[derive(Clone)]
pub struct KnowledgeBasePaulGraham;

#[async_trait::async_trait]
impl Tool for KnowledgeBasePaulGraham {
    fn name(&self) -> &'static str {
        "knowledge_base_paul_graham"
    }
    fn description(&self) -> &'static str {
        "Knowledge base containing information on Paul Graham"
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
        let kb_url = "http://127.0.0.1:8001/knowledge_base/1/similar";
        let args: Value = serde::json::from_str(&args)?;
        let content = args["content"].as_str().unwrap();
        let body = json!({"text":content, "num_results":3});
        let response = client.post(kb_url).json(&body).send().await?;
        let result = response.json::<Vec<Content>>().await?;
        Ok(json!(result))
    }
}
/*
#[derive(Clone)]
pub struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "knowledge_base"
    }
    fn description(&self) -> &'static str {
        "Get data from a knowledge base"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The message content required to search the knowledge base",
                },
            },
            "required": ["content"],
        })
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let args: Value = serde::json::from_str(&args)?;
        //args should have content key, see parameters
        Ok(json!({"echo":args}))
    }
}
*/
