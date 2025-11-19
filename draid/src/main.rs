mod api;
mod auth;
mod config;
mod dbtracing;
mod embedding;
mod kb_tools;
mod llm;
mod mcp_tools;
mod models;
mod prompts;
mod psql_memory;
mod psql_users;
mod psql_vectors;
mod tools;

use api::{Api, helper_ws_handler, tutor_ws_handler};
use auth::{JwtMiddleware, WSMiddleware};
use config::Config;
use dbtracing::create_logging;
use embedding::EmbeddingClient;
use models::get_bots;
use poem::middleware::Tracing;
use poem::{EndpointExt, Route, listener::TcpListener};
use poem_openapi::OpenApiService;
use psql_users::create_init_admin_user;
use psql_vectors::write_knowledge_base;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tools::{AddTool, TimeTool, Tool};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //environment variables
    let open_ai_compatable_endpoint_chat = env::var("OPEN_AI_COMPATABLE_ENDPOINT_CHAT")
        .unwrap_or_else(|_e| "http://localhost:11434".to_string());

    let open_ai_compatable_endpoint_embedding = env::var("OPEN_AI_COMPATABLE_ENDPOINT_EMBEDDING")
        .unwrap_or_else(|_e| "http://localhost:11434".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap().into_bytes();
    let psql_url = env::var("PSQL_DATABASE_URL").unwrap();
    let init_admin_password = env::var("INIT_ADMIN_PASSWORD").unwrap();
    let port = env::var("PORT").unwrap_or_else(|_e| "3000".to_string());
    let address = env::var("ADDRESS").unwrap_or_else(|_e| "0.0.0.0".to_string());
    let actual_endpoint_for_swagger =
        env::var("HOSTNAME").unwrap_or_else(|_e| "http://localhost:3000".to_string());
    let kb_endpoint = env::var("KNOWLEDGE_BASE_ENDPOINT")
        .unwrap_or_else(|_e| "http://127.0.0.1:3000".to_string());

    let default_raw_tool_config = r#"{
        "kb": [
            {
                "name": "recipes",
                "num_results": 3
            },
            {
                "name": "gardening",
                "num_results": 3
            }
        ],
        "mcp": [
            {
                "name": "devin",
                "description": "Devin allows searching code repositories.  No auth required.  Both owner and repo must be provided to the function.",
                "url": "https://mcp.deepwiki.com/mcp",
                "mcp_type": "stream"
            }
        ]
    }"#;
    let tool_config_raw =
        env::var("TOOL_CONFIG").unwrap_or_else(|_e| default_raw_tool_config.to_string());

    //db setup
    let pool = PgPool::connect(&psql_url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations.");
    create_init_admin_user(init_admin_password, &pool).await?;

    //llm and embedding setup
    let model_name = "hf.co/Qwen/Qwen3-4B-GGUF:latest";
    let embedding_client = Arc::new(EmbeddingClient::new(
        //"bge-m3:567m".to_string(),
        "hf.co/mixedbread-ai/mxbai-embed-large-v1".to_string(),
        &open_ai_compatable_endpoint_embedding,
    ));

    //tools
    let tool_config: Config = serde_json::from_str(&tool_config_raw)?;
    let kb_arcs = kb_tools::get_tools(tool_config.kb, &kb_endpoint);
    for kb_arc in kb_arcs.iter() {
        match write_knowledge_base(&kb_arc.name(), &pool).await {
            Ok(result) => println!(
                "Created knowledge base {} with index {}",
                kb_arc.name(),
                result
            ),
            Err(e) => println!("Failed to create knowledge base: {}", e),
        }
    }

    let (mcp_arcs, _servers) = mcp_tools::get_tools_and_servers(tool_config.mcp).await?;

    let mut helper_tools: Vec<Arc<dyn Tool + Send + Sync>> =
        vec![Arc::new(AddTool::new()), Arc::new(TimeTool::new())];
    helper_tools.extend(kb_arcs);
    helper_tools.extend(mcp_arcs);

    //bots
    let bots = Arc::new(
        get_bots(
            model_name.to_string(),
            open_ai_compatable_endpoint_chat,
            helper_tools,
        )
    );

    //logging setup
    //this is a future, can be awaited but then blocks everything
    let _logging_handle = create_logging(&pool);

    //API setup
    let api_service = OpenApiService::new(Api, "Draid", "1.0").server(actual_endpoint_for_swagger);
    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/", api_service.with(JwtMiddleware)) //what about login?
        .at("/ws/tutor", tutor_ws_handler.with(WSMiddleware))
        .at("/ws/helper", helper_ws_handler.with(WSMiddleware))
        .nest("/docs", ui)
        .with(Tracing)
        .data(jwt_secret)
        .data(pool)
        .data(bots)
        .data(embedding_client);
    poem::Server::new(TcpListener::bind(format!("{}:{}", address, port)))
        .run(app)
        .await?;
    Ok(())
}
