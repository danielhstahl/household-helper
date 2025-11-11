use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KB {
    pub name: String,
    pub num_results: i32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum MCPType {
    STREAM,
    SSE,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MCP {
    pub name: String,
    pub description: String,
    pub url: String,
    pub mcp_type: MCPType,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub kb: Vec<KB>,
    pub mcp: Vec<MCP>,
}
