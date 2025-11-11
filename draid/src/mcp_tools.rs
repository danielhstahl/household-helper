use crate::config::{MCP, MCPType};
use crate::tools::Tool;
use futures::future;
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParam, Tool as McpTool},
    service::{RunningService, ServerSink},
};
use serde_json::Value;
use std::sync::Arc;

async fn get_server_and_tools_for_single_mcp(
    mcp_type: MCPType,
    url: String,
) -> anyhow::Result<(Vec<McpTool>, RunningService<RoleClient, ()>)> {
    let server = match mcp_type {
        MCPType::STREAM => {
            let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(url);
            ().serve(transport).await
        }
        MCPType::SSE => {
            let transport = rmcp::transport::sse_client::SseClientTransport::start(url).await?;
            ().serve(transport).await
        }
    }?;
    let tools = server.peer().list_all_tools().await?;
    Ok((tools, server))
}

pub async fn get_tools_and_servers(
    mcp_configs: Vec<MCP>,
) -> anyhow::Result<(
    Vec<Arc<dyn Tool + Send + Sync>>,
    Vec<RunningService<RoleClient, ()>>,
)> {
    let futures: Vec<_> = mcp_configs
        .into_iter()
        .map(|mcp_config| async move {
            let (tools, server) = get_server_and_tools_for_single_mcp(
                mcp_config.mcp_type.clone(),
                mcp_config.url.clone(),
            )
            .await?;
            let tools: Vec<Arc<dyn Tool + Send + Sync>> = tools
                .into_iter()
                .map(|tool| {
                    Arc::new(MCPTool::new(
                        tool,
                        server.peer().clone(),
                        mcp_config.clone(),
                    )) as Arc<dyn Tool + Send + Sync>
                })
                .collect();
            Ok::<_, anyhow::Error>((tools, server))
        })
        .collect();
    let results = future::try_join_all(futures).await?;

    let (tools, servers): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    let tools: Vec<_> = tools
        .into_iter()
        .map(|tool_v| tool_v.into_iter())
        .flatten()
        .collect();
    //servers is only returned so that it doesn't get cleaned up when this function completes
    Ok((tools, servers))
}

#[derive(Clone)]
pub struct MCPTool {
    tool: McpTool,
    server: ServerSink,
    name: String,
    description: String,
}

impl MCPTool {
    pub fn new(tool: McpTool, server: ServerSink, mcp_config: MCP) -> Self {
        Self {
            tool,
            server,
            name: mcp_config.name,
            description: mcp_config.description,
        }
    }
}

// Generated impl block, replacing placeholders with parsed values
#[async_trait::async_trait]
impl Tool for MCPTool {
    fn name(&self) -> &String {
        // Use the generated static string literal
        &self.name
    }
    fn description(&self) -> &String {
        &self.description
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(&self.tool.input_schema).unwrap_or(serde_json::json!({}))
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let args: Value = serde_json::from_str(&args)?;
        let arguments = match args {
            Value::Object(map) => Some(map),
            _ => None,
        };
        let call_result = self
            .server
            .call_tool(CallToolRequestParam {
                name: self.tool.name.clone(),
                arguments,
            })
            .await?;
        let json_value = serde_json::to_value(call_result)?;
        Ok(json_value)
    }
}
