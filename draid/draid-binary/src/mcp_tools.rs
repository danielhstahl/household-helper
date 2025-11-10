//use crate::tools::Tool;
use rmcp::{
    RoleClient, ServiceExt,
    model::{/*CallToolRequestParam,*/ Tool as McpTool},
    service::RunningService,
};

pub async fn get_server_and_tools(
    mcp_type: &str,
    url: &str,
) -> anyhow::Result<(Vec<McpTool>, RunningService<RoleClient, ()>)> {
    let server = match mcp_type {
        "stream" => {
            let transport =
                rmcp::transport::StreamableHttpClientTransport::from_uri(url.to_string());
            ().serve(transport).await
        }
        "sse" => {
            let transport =
                rmcp::transport::sse_client::SseClientTransport::start(url.to_string()).await?;
            ().serve(transport).await
        }
        _ => Err(rmcp::service::ClientInitializeError::ExpectedInitResponse(
            None,
        )),
    }?;
    let tools = server.peer().list_all_tools().await?;
    Ok((tools, server))
}
