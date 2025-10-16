mod auth;
mod dbtracing;
mod embedding;
mod llm;
mod prompts;
mod psql_memory;
mod psql_users;
mod psql_vectors;
mod tools;

use auth::UserIdentification;
use dbtracing::{HistogramIncrement, SpanToolUse, create_logging, get_histogram, get_tool_use};
use embedding::{EmbeddingClient, get_embeddings};
use kb_tool_macro::kb;
use llm::{Bot, chat_with_tools};
use poem::error::InternalServerError;
//use poem::web::Query;
use poem::{
    EndpointExt, Error, Request, Result, Route, http::StatusCode, listener::TcpListener,
    middleware::AddData, web::Data, /*web::Json,*/ web::Path,
};
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, Enum};
use poem_openapi::{
    Object, OpenApi, OpenApiService, SecurityScheme,
    auth::{ApiKey, Basic, BearerAuthorization},
    param::Query,
    payload::PlainText,
};
use prompts::HELPER_PROMPT;
use prompts::TUTOR_PROMPT;
use psql_memory::{MessageResult, PsqlMemory, manage_chat_interaction};
use psql_users::{Role, SessionDB, UserRequest, UserResponse, create_user};
use psql_vectors::{
    KnowledgeBase, get_docs_with_similar_content, get_knowledge_base, get_knowledge_bases,
    write_chunk_content, write_document, write_knowledge_base,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, Type, query, types::chrono};
use std::sync::{Arc, Mutex};
use std::{env, fmt};
use tokio::sync::mpsc;
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

/// Basic authorization
///
/// - User: `test`
/// - Password: `123456`
#[derive(SecurityScheme)]
#[oai(ty = "basic")]
struct MyBasicAuthorization(Basic);

/// ApiKey authorization
/*#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_name = "X-API-Key",
    key_in = "header",
    checker = "api_checker"
)]
struct MyApiKeyAuthorization(User);

async fn api_checker(req: &Request, api_key: ApiKey) -> Option<User> {
    let server_key = req.data::<ServerKey>().unwrap();
    VerifyWithKey::<User>::verify_with_key(api_key.key.as_str(), server_key).ok()
}*/

#[derive(Object)]
struct LoginRequest {
    username: String,
}

struct Bots {
    helper_bot: Bot,
    tutor_bot: Bot,
}

async fn get_bots(
    model_name: String,
    open_ai_compatable_endpoint: String,
    pool: &PgPool,
) -> Result<Bots, Error> {
    let kb_arcs: Vec<Arc<dyn Tool + Send + Sync>> = vec![kb!("recipes", 3), kb!("gardening", 3)];
    for kb_arc in kb_arcs.iter() {
        match write_knowledge_base(kb_arc.name(), pool).await {
            Ok(result) => println!(
                "Created knowledge base {} with index {}",
                kb_arc.name(),
                result
            ),
            Err(e) => println!("Failed to create knowledge base: {}", e),
        }
    }
    let mut helper_tools: Vec<Arc<dyn Tool + Send + Sync>> =
        vec![Arc::new(AddTool), Arc::new(TimeTool)];
    helper_tools.extend(kb_arcs);

    let bots = Bots {
        helper_bot: Bot::new(
            model_name.clone(),
            HELPER_PROMPT,
            &open_ai_compatable_endpoint,
            //recommended for qwen, see eg https://huggingface.co/Qwen/Qwen3-4B-GGUF#best-practices
            Some(0.6),  //temperature
            Some(1.5),  //presence penalty
            Some(0.95), //top_p
            Some(helper_tools),
        ),
        tutor_bot: Bot::new(
            model_name,
            TUTOR_PROMPT,
            &open_ai_compatable_endpoint,
            //recommended for qwen, see eg https://huggingface.co/Qwen/Qwen3-4B-GGUF#best-practices
            Some(0.6),  //temperature
            Some(1.5),  //presence penalty
            Some(0.95), //top_p
            None,       //no tools
        ),
    };
    Ok(bots)
}

async fn chat_with_bot(bot: &Bot, psql_memory: PsqlMemory, prompt: &str) -> Result<()> {
    let messages = psql_memory.messages().await.map_err(InternalServerError)?;
    let tx_persist_message = manage_chat_interaction(&prompt, psql_memory)
        .await
        .map_err(InternalServerError)?;

    let (tx, mut rx) = mpsc::channel::<String>(100);

    let span_id = Uuid::new_v4().to_string();

    //let remote_prompt = prompt.to_string();

    //bot may not need to be cloned
    chat_with_tools(&bot, tx, &messages, &prompt, span_id)
        .instrument(span!(
            Level::INFO,
            "chat_with_tools",
            endpoint = "query",
            tool_use = false
        ))
        .await
        .map_err(|e| InternalServerError(e.to_string()))?;
    //frustrating that I'm cloning...I tried to get bot and prompt to be efficient
    /*tokio::spawn(async move {
        if let Err(e) = chat_with_tools(bot, tx, &messages, &remote_prompt, span_id)
            .instrument(span!(
                Level::INFO,
                "chat_with_tools",
                endpoint = "query",
                tool_use = false
            ))
            .await
        {
            eprintln!("chat_with_tools exploded: {}", e);
        }
    });
    Ok(TextStream! {
        while let Some(chunk) = rx.recv().await {
            if let Err(e) = tx_persist_message.send(chunk.clone()).await {
                eprintln!("Failed to send chunk to background task: {}", e);
            }
            yield chunk
        }
    })*/
    Ok(())
}

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
pub enum SuccessResponse {
    // Status 200: Success
    #[oai(status = 200)]
    Success(Json<StatusResponse>),

    // Status 500: Internal Server Error
    #[oai(status = 500)]
    InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
pub enum UsersResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<UserResponse>>),

    #[oai(status = 200)]
    SuccessSingle(Json<UserResponse>),

    // Status 500: Internal Server Error
    #[oai(status = 500)]
    InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
pub enum SessionResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<SessionDB>>),

    #[oai(status = 200)]
    SuccessSingle(Json<SessionDB>),

    // Status 500: Internal Server Error
    #[oai(status = 500)]
    InternalError(Json<ApiError>),
}

#[derive(ApiResponse)]
pub enum MessageResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<MessageResult>>),

    #[oai(status = 200)]
    SuccessSingle(Json<MessageResult>),

    // Status 500: Internal Server Error
    #[oai(status = 500)]
    InternalError(Json<ApiError>),
}

#[derive(Deserialize, Object)]
struct Prompt {
    text: String,
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

struct Api;
#[poem_grants::open_api]
#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
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
    #[oai(path = "/user/:id", method = "post")]
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
        auth: MyBasicAuthorization,
    ) -> Result<Json<AuthResponse>> {
        psql_users::authenticate_user(&auth.0.username, &auth.0.password, pool)
            .await
            .map_err(|_e| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let access_token = auth::create_token(auth.0.username.to_string(), &jwt_secret)
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

    #[protect("Role::Helper", ty = "Role")]
    #[oai(path = "/helper?:session_id", method = "post")]
    async fn helper(
        &self,
        Query(session_id): Query<Uuid>,
        prompt: Json<Prompt>,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
        bots: Data<&Bots>,
    ) -> Result<()> {
        //websocket
        let psql_memory = PsqlMemory::new(100, session_id, user.id, pool.clone());
        chat_with_bot(&bots.helper_bot, psql_memory, &prompt.text).await
    }
    #[protect("Role::Tutor", ty = "Role")]
    #[oai(path = "/tutor?:session_id", method = "post")]
    async fn tutor(
        &self,
        Query(session_id): Query<Uuid>,
        prompt: Json<Prompt>,
        Data(user): Data<&UserIdentification>, //attached from auth middleware
        Data(pool): Data<&PgPool>,
        bots: Data<&Bots>,
    ) -> Result<()> {
        //websocket
        let psql_memory = PsqlMemory::new(100, session_id, user.id, pool.clone());
        chat_with_bot(&bots.tutor_bot, psql_memory, &prompt.text).await
    }

    #[protect("Role::Admin", ty = "Role")]
    #[oai(path = "/telemetry/latency/:endpoint", method = "get")]
    async fn histogram(
        &self,
        Path(endpoint): Path<String>,
        Data(pool): Data<&PgPool>,
    ) -> Result<Json<Vec<HistogramIncrement>>> {
        //websocket
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
        //websocket
        let results = get_tool_use(pool, &endpoint)
            .await
            .map_err(InternalServerError)?;
        Ok(Json(results))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let open_ai_compatable_endpoint_chat = env::var("OPEN_AI_COMPATABLE_ENDPOINT_CHAT")
        .unwrap_or_else(|_e| "http://localhost:11434".to_string());

    let open_ai_compatable_endpoint_embedding = env::var("OPEN_AI_COMPATABLE_ENDPOINT_EMBEDDING")
        .unwrap_or_else(|_e| "http://localhost:11434".to_string());

    let model_name = "hf.co/Qwen/Qwen3-4B-GGUF:latest";

    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();
    let jwt_secret = env::var("JWT_SECRET").unwrap().into_bytes();
    let psql_url = env::var("PSQL_DATABASE_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

    let logging_handle = create_logging(&pool).await?;
    let bots = get_bots(
        model_name.to_string(),
        open_ai_compatable_endpoint_chat,
        &pool,
    )
    .await?;
    let embedding_client = Arc::new(EmbeddingClient::new(
        //"bge-m3:567m".to_string(),
        "hf.co/mixedbread-ai/mxbai-embed-large-v1".to_string(),
        &open_ai_compatable_endpoint_embedding,
    ));

    let bots_arc = Arc::new(bots);

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/", ui)
        .data(jwt_secret)
        .data(pool)
        .data(bots_arc)
        .data(embedding_client);

    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;
    Ok(())
}
