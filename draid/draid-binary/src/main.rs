mod auth;
mod dbtracing;
mod embedding;
mod llm;
mod prompts;
mod psql_users;
mod psql_vectors;
mod tools;

use dbtracing::create_logging;
use embedding::{EmbeddingClient, get_embeddings};
use kb_tool_macro::kb;
use llm::{Bot, chat_with_tools};
use poem::{
    Error, Request, Result, Route, http::StatusCode, listener::TcpListener, middleware::AddData,
    web::Data,
};
use poem_openapi::payload::Json;
use poem_openapi::{
    Object, OpenApi, OpenApiService, SecurityScheme,
    auth::{ApiKey, Basic, BearerAuthorization},
    param::Query,
    payload::PlainText,
};
use prompts::HELPER_PROMPT;
use prompts::TUTOR_PROMPT;
use psql_users::{Role, SessionDB, UserRequest, UserResponse, create_user};
use psql_vectors::{
    KnowledgeBase, get_docs_with_similar_content, get_knowledge_base, get_knowledge_bases,
    write_chunk_content, write_document, write_knowledge_base,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, Type, query, types::chrono};
use std::env;
use std::sync::{Arc, Mutex};
use tools::{AddTool, TimeTool, Tool};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
}

#[derive(Debug, Serialize)]
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
    db: &PgConnection,
) -> Result<Bots, Error> {
    let kb_arcs: Vec<Arc<dyn Tool + Send + Sync>> = vec![kb!("recipes", 3), kb!("gardening", 3)];
    for kb_arc in kb_arcs.iter() {
        match write_knowledge_base(kb_arc.name(), &mut *db).await {
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

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
    #[oai(path = "/login", method = "post")]
    async fn login(
        &self,
        db: Data<&PgConnection>,
        jwt_secret: Data<&Vec<u8>>,
        auth: MyBasicAuthorization,
        req: Json<LoginRequest>,
    ) -> Result<AuthResponse> {
        psql_users::authenticate_user(&auth.0.username, &auth.0.password, &mut db)
            .await
            .map_err(|_e| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let access_token = auth::create_token(auth.0.username.to_string(), &jwt_secret)
            .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        Ok(Json(AuthResponse { access_token }))
    }

    /// This API returns the currently logged in user.
    #[oai(path = "/hello", method = "get")]
    async fn hello(&self, auth2: MyBasicAuthorization) -> Result<PlainText<String>> {
        if auth2.0.username != "test" || auth2.0.password != "123456" {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        Ok(PlainText(auth2.0.username))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
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
    let pool = PgPoolOptions::new()
        .max_connections(100) //one hundred connections to start with
        .connect(psql_url)
        .await?;

    let logging_handle = create_logging(&pool);
    let bots = get_bots(model_name, open_ai_compatable_endpoint_chat, &pool).await?;
    let embedding_client = Arc::new(EmbeddingClient::new(
        //"bge-m3:567m".to_string(),
        "hf.co/mixedbread-ai/mxbai-embed-large-v1".to_string(),
        &open_ai_compatable_endpoint_embedding,
    ));

    let app = Route::new()
        .nest("/api", api_service)
        .data(jwt_secret)
        .data(pool)
        .data(bots)
        .data(embedding_client)
        //.with(AddData::new(pool))
        .nest("/", ui);

    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
