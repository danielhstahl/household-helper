mod auth;
mod config;
mod dbtracing;
mod embedding;
mod kb_tools;
mod llm;
mod mcp_tools;
mod prompts;
mod psql_memory;
mod psql_users;
mod psql_vectors;
mod tools;

use auth::{JwtMiddleware, UserIdentification, WSMiddleware};
use chrono::{DateTime, Utc};
use config::Config;
use dbtracing::{HistogramIncrement, SpanToolUse, create_logging, get_histogram, get_tool_use};
use embedding::{EmbeddingClient, get_embeddings, ingest_content};
use futures::future::{BoxFuture, FutureExt};
use futures::{SinkExt, StreamExt};
use llm::{Bot, chat_with_tools};
use poem::error::InternalServerError;
use poem::middleware::Tracing;
use poem::web::websocket::{Message, WebSocket, WebSocketStream};
use poem::web::{Multipart, Query as WsQuery};
use poem::{
    EndpointExt, Error, Result, Route, http::StatusCode, listener::TcpListener, web::Data,
    web::Form, web::Path,
};
use poem::{IntoResponse, handler};
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, Enum};
use poem_openapi::{Object, OpenApi, OpenApiService};
use prompts::HELPER_PROMPT;
use prompts::TUTOR_PROMPT;
use psql_memory::{MessageResult, PsqlMemory, write_ai_message, write_human_message};
use psql_users::{Role, SessionDB, UserResponse, create_init_admin_user};
use psql_vectors::{
    KnowledgeBase, get_docs_with_similar_content, get_knowledge_base, get_knowledge_bases,
    write_knowledge_base,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::{env, fmt};
use tools::{AddTool, TimeTool, Tool};
use tracing::{Instrument, Level, span};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
}

#[derive(Debug, Serialize, Object)]
struct AuthResponse {
    access_token: String,
}

#[derive(Debug, Deserialize, Object)]
struct AuthRequest {
    username: String,
    password: String,
}

struct Bots {
    helper_bot: Arc<Bot>,
    tutor_bot: Arc<Bot>,
}

async fn get_bots(
    model_name: String,
    open_ai_compatable_endpoint: String,
    helper_tools: Vec<Arc<dyn Tool + Send + Sync>>,
) -> anyhow::Result<Bots> {
    let bots = Bots {
        helper_bot: Arc::new(Bot::new(
            model_name.clone(),
            HELPER_PROMPT,
            &open_ai_compatable_endpoint,
            //recommended for qwen, see eg https://huggingface.co/Qwen/Qwen3-4B-GGUF#best-practices
            Some(0.6),  //temperature
            Some(1.5),  //presence penalty
            Some(0.95), //top_p
            Some(helper_tools),
        )),
        tutor_bot: Arc::new(Bot::new(
            model_name,
            TUTOR_PROMPT,
            &open_ai_compatable_endpoint,
            //recommended for qwen, see eg https://huggingface.co/Qwen/Qwen3-4B-GGUF#best-practices
            Some(0.6),  //temperature
            Some(1.5),  //presence penalty
            Some(0.95), //top_p
            None,       //no tools
        )),
    };
    Ok(bots)
}

#[derive(Debug)]
pub struct NoData {
    msg: String,
}
impl fmt::Display for NoData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for NoData {}

#[derive(Debug)]
pub struct LLMError {
    msg: String,
}
impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for LLMError {}

#[derive(Debug, Serialize, Enum)]
enum ResponseStatus {
    Success,
    //Failure,
}

#[derive(Debug, Serialize, Object)]
struct StatusResponse {
    status: ResponseStatus,
}

// 1. Define the Error Payload
#[derive(Debug, Serialize, Object)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

#[derive(ApiResponse)]
enum SuccessResponse {
    // Status 200: Success
    #[oai(status = 200)]
    Success(Json<StatusResponse>),
    // Status 500: Internal Server Error
    //#[oai(status = 500)]
    //InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
enum UsersResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<UserResponse>>),

    #[oai(status = 200)]
    SuccessSingle(Json<UserResponse>),
    // Status 500: Internal Server Error
    //#[oai(status = 500)]
    //InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
enum SessionResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<SessionDB>>),

    #[oai(status = 200)]
    SuccessSingle(Json<SessionDB>),
    // Status 500: Internal Server Error
    //#[oai(status = 500)]
    //InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
enum MessageResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<MessageResult>>),
    /*#[oai(status = 200)]
    SuccessSingle(Json<MessageResult>),

    // Status 500: Internal Server Error
    #[oai(status = 500)]
    InternalError(Json<ApiError>),*/
}

#[derive(Deserialize, Object)]
struct Prompt {
    text: String,
}

#[derive(Deserialize)] // No deny_unknown_fields; let token slide, it's inert here
struct SessionQuery {
    session_id: Uuid,
}
#[derive(Object, Serialize)]
struct UploadResponse {
    filename: String,
    size: usize,
    hash: String, // e.g., blake3 for that cyberpunk veracity
}

pub fn handle_chat_session(
    bot_ref: &Arc<Bot>,
    session_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> impl Fn(WebSocketStream) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static {
    let bot = bot_ref.clone(); //its weird I need so many clones...but they are cheap (on Arcs)
    let pool = pool.clone();
    move |mut socket: WebSocketStream| -> BoxFuture<'static, Result<()>> {
        let bot = bot.clone();
        let pool = pool.clone();
        async move {
            while let Some(Ok(Message::Text(prompt))) = &mut socket.next().await {
                // recreate each request.  This is as "performant" as
                // standard rest request was in previous approach
                let psql_memory = PsqlMemory::new(100, session_id, user_id, pool.clone());
                let messages = psql_memory.messages().await.map_err(InternalServerError)?;
                write_human_message(prompt.clone(), &psql_memory)
                    .await
                    .map_err(InternalServerError)?;
                let span_id = Uuid::new_v4().to_string();
                //chat_with_tools produces each token in the stream to the websocket
                let full_message = chat_with_tools(&bot, &mut socket, &messages, &prompt, span_id)
                    .instrument(span!(
                        Level::INFO,
                        "chat_with_tools",
                        endpoint = "query",
                        tool_use = false
                    ))
                    .await
                    .map_err(|e| {
                        let utc: DateTime<Utc> = Utc::now();
                        println!("{}-{:?}", utc, e); //log to stdout as well
                        InternalServerError(LLMError { msg: e.to_string() })
                    })?;
                write_ai_message(full_message, &psql_memory)
                    .await
                    .map_err(InternalServerError)?;
                socket
                    .send(Message::Close(None))
                    .await
                    .map_err(InternalServerError)?;
            }
            Ok(())
        }
        .boxed()
    }
}

#[poem_grants::protect("Role::Tutor", ty = "Role")]
#[handler]
async fn tutor_ws_handler(
    WsQuery(SessionQuery { session_id }): WsQuery<SessionQuery>,
    Data(bot): Data<&Arc<Bots>>,
    Data(pool): Data<&PgPool>,
    Data(user): Data<&UserIdentification>, //attached from auth middleware
    ws: WebSocket,
) -> Result<poem::Response> {
    let ws_upgrade = ws.on_upgrade(handle_chat_session(
        &bot.tutor_bot,
        session_id,
        user.id,
        pool,
    ));
    Ok(ws_upgrade.into_response())
}

#[poem_grants::protect("Role::Helper", ty = "Role")]
#[handler]
async fn helper_ws_handler(
    WsQuery(SessionQuery { session_id }): WsQuery<SessionQuery>,
    Data(bot): Data<&Arc<Bots>>,
    Data(pool): Data<&PgPool>,
    Data(user): Data<&UserIdentification>, //attached from auth middleware
    ws: WebSocket,
) -> Result<poem::Response> {
    let ws_upgrade = ws.on_upgrade(handle_chat_session(
        &bot.helper_bot,
        session_id,
        user.id,
        pool,
    ));
    Ok(ws_upgrade.into_response())
}

#[derive(Deserialize, Object)]
struct PromptKb {
    text: String,
    num_results: i16,
}
async fn similar_content(
    kb_id: i64,
    prompt: Json<PromptKb>,
    pool: &PgPool,
    client: &EmbeddingClient,
) -> Result<Json<Vec<String>>> {
    let embeddings = get_embeddings(&client, &prompt.text)
        .await
        .map_err(InternalServerError)?;
    let result = get_docs_with_similar_content(kb_id, embeddings, prompt.num_results, pool)
        .await
        .map_err(InternalServerError)?;
    Ok(Json(result))
}

struct Api;
#[poem_grants::open_api]
#[OpenApi]
impl Api {
    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/user", method = "post")]
    async fn new_user(
        &self,
        user: Json<psql_users::UserRequest>,
        Data(pool): Data<&PgPool>,
    ) -> Result<SuccessResponse> {
        psql_users::create_user(&user, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SuccessResponse::Success(Json(StatusResponse {
            status: ResponseStatus::Success,
        })))
    }
    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/user/:id", method = "delete")]
    async fn delete_user(
        &self,
        Path(id): Path<Uuid>,
        Data(pool): Data<&PgPool>,
    ) -> Result<SuccessResponse> {
        psql_users::delete_user(&id, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SuccessResponse::Success(Json(StatusResponse {
            status: ResponseStatus::Success,
        })))
    }

    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/user/:id", method = "patch")]
    async fn update_user(
        &self,
        Path(id): Path<Uuid>,
        user: Json<psql_users::UserRequest>,
        Data(pool): Data<&PgPool>,
    ) -> Result<SuccessResponse> {
        psql_users::patch_user(&id, &user, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SuccessResponse::Success(Json(StatusResponse {
            status: ResponseStatus::Success,
        })))
    }

    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/user", method = "get")]
    async fn get_users(&self, Data(pool): Data<&PgPool>) -> Result<UsersResponse> {
        let users = psql_users::get_all_users(pool)
            .await
            .map_err(InternalServerError)?;
        Ok(UsersResponse::SuccessMultiple(Json(users)))
    }

    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/user/me", method = "get")]
    async fn get_user(
        &self,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<UsersResponse> {
        let user = psql_users::get_user(&user.username, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(UsersResponse::SuccessSingle(Json(user)))
    }

    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/session", method = "post")]
    async fn new_session(
        &self,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<SessionResponse> {
        let session = psql_users::create_session(&user.id, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SessionResponse::SuccessSingle(Json(session)))
    }

    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/session/:session_id", method = "delete")]
    async fn delete_session(
        &self,
        Path(session_id): Path<Uuid>,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<SuccessResponse> {
        psql_users::delete_session(&session_id, &user.id, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SuccessResponse::Success(Json(StatusResponse {
            status: ResponseStatus::Success,
        })))
    }
    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/session", method = "get")]
    async fn get_sessions(
        &self,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<SessionResponse> {
        let sessions = psql_users::get_all_sessions(&user.id, pool)
            .await
            .map_err(InternalServerError)?;
        Ok(SessionResponse::SuccessMultiple(Json(sessions)))
    }

    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/session/recent", method = "get")]
    async fn latest_session(
        &self,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<SessionResponse> {
        let session = psql_users::get_most_recent_session(&user.id, pool)
            .await
            .map_err(InternalServerError)?;
        let session = session.ok_or_else(|| {
            InternalServerError(NoData {
                msg: "No session".to_string(),
            })
        })?;
        Ok(SessionResponse::SuccessSingle(Json(session)))
    }

    #[oai(path = "/login", method = "post")]
    async fn login(
        &self,
        Data(pool): Data<&PgPool>,
        Data(jwt_secret): Data<&Vec<u8>>,
        Form(auth): Form<AuthRequest>,
    ) -> Result<Json<AuthResponse>> {
        psql_users::authenticate_user(&auth.username, &auth.password, pool)
            .await
            .map_err(|_e| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let access_token = auth::create_token(auth.username, &jwt_secret)
            .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        Ok(Json(AuthResponse { access_token }))
    }

    #[protect(any("Role::Admin", "Role::Tutor", "Role::Helper"), ty = "Role")]
    #[oai(path = "/messages/:session_id", method = "get")]
    async fn get_messages(
        &self,
        Path(session_id): Path<Uuid>,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
    ) -> Result<MessageResponse> {
        let psql_memory = PsqlMemory::new(100, session_id, user.id, pool.clone());
        let messages = psql_memory.messages().await.map_err(InternalServerError)?;
        Ok(MessageResponse::SuccessMultiple(Json(messages)))
    }

    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/telemetry/latency/:endpoint", method = "get")]
    async fn histogram(
        &self,
        Path(endpoint): Path<String>,
        Data(pool): Data<&PgPool>,
    ) -> Result<Json<Vec<HistogramIncrement>>> {
        let results = get_histogram(pool, &endpoint)
            .await
            .map_err(InternalServerError)?;
        Ok(Json(results))
    }
    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/telemetry/tools/:endpoint", method = "get")]
    async fn tool_use(
        &self,
        Path(endpoint): Path<String>,
        Data(pool): Data<&PgPool>,
    ) -> Result<Json<Vec<SpanToolUse>>> {
        let results = get_tool_use(pool, &endpoint)
            .await
            .map_err(InternalServerError)?;
        Ok(Json(results))
    }

    #[oai(path = "/knowledge_base/:kb/similar", method = "post")]
    async fn similar_kb_by_name(
        &self,
        Path(kb): Path<String>,
        prompt: Json<PromptKb>,
        Data(pool): Data<&PgPool>,
        Data(client): Data<&Arc<EmbeddingClient>>,
    ) -> Result<Json<Vec<String>>> {
        let KnowledgeBase { id, .. } = get_knowledge_base(&kb, pool)
            .await
            .map_err(InternalServerError)?;
        similar_content(id, prompt, pool, client).await
    }
    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/knowledge_base", method = "get")]
    async fn get_kbs(&self, Data(pool): Data<&PgPool>) -> Result<Json<Vec<KnowledgeBase>>> {
        Ok(Json(
            get_knowledge_bases(pool)
                .await
                .map_err(InternalServerError)?,
        ))
    }

    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/knowledge_base/:kb/ingest", method = "post")]
    async fn upload_file(
        &self,
        Path(kb): Path<String>,
        mut multipart: Multipart,
        Data(client): Data<&Arc<EmbeddingClient>>,
        Data(pool): Data<&PgPool>,
    ) -> Result<Json<UploadResponse>> {
        let KnowledgeBase { id, .. } = get_knowledge_base(&kb, pool)
            .await
            .map_err(InternalServerError)?;
        let mut filename = None;
        let mut data = Vec::new();
        while let Some(field) = multipart.next_field().await? {
            //this MUST match the client-side name: formData.append("file", file)
            if field.name() == Some("file") {
                filename = field.file_name().map(String::from);
                data = field.bytes().await?; // Own the bytes; drop cleanses the void
            }
        }
        if data.is_empty() {
            return Err(Error::from_status(StatusCode::BAD_REQUEST)); // Bad input? *Thrownness* into error.
        }
        let data_size = data.len();
        let hash = ingest_content(id, pool, data, client)
            .await
            .map_err(|e| InternalServerError(NoData { msg: e.to_string() }))?;
        let resp = UploadResponse {
            filename: filename.unwrap_or("anonymous_sprawl".to_string()),
            size: data_size,
            hash,
        };

        Ok(Json(resp)) // Serializes to JSON; spec gen handles the rest
    }
}

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
        .await?,
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
