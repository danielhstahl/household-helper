use rocket::{
    serde,
    serde::json::{Value, json},
};
use sqlx::types::chrono;
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

#[derive(Clone)]
pub struct TimeTool;

#[async_trait::async_trait]
impl Tool for TimeTool {
    fn name(&self) -> &'static str {
        "time"
    }
    fn description(&self) -> &'static str {
        "Get current time"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
            },
            "required": [],
        })
    }
    async fn invoke(&self, _args: String) -> anyhow::Result<Value> {
        let result = chrono::Local::now();
        Ok(json!({"result":result}))
    }
}
