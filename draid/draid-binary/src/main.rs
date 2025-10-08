#[macro_use]
extern crate rocket;
mod auth;
mod dbtracing;
mod embedding;
mod llm;
mod prompts;
mod psql_memory;
mod psql_users;
mod psql_vectors;
mod tools;

use dbtracing::{
    AsyncDbWorker, HistogramIncrement, PSqlLayer, SpanToolUse, get_histogram, get_tool_use,
    run_async_worker,
};
use embedding::{EmbeddingClient, get_embeddings};
use futures::stream::{self, StreamExt};
use kb_tool_macro::kb;
use llm::{Bot, chat_with_tools};
use prompts::HELPER_PROMPT;
use prompts::TUTOR_PROMPT;
use psql_memory::{MessageResult, PsqlMemory, manage_chat_interaction};
use psql_users::{Role, SessionDB, UserRequest, UserResponse, create_user};
use psql_vectors::{
    KnowledgeBase, SimilarContent, get_knowledge_base, get_knowledge_bases, get_similar_content,
    write_document, write_knowledge_base, write_single_content,
};
use reqwest::Client as HttpClient;
use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{self, AdHoc};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::response::stream::TextStream;
use rocket::serde::{
    Deserialize, Serialize, json,
    json::{Json, Value, json},
    uuid::Uuid,
};
use rocket::tokio::sync::mpsc::{self};
use rocket::{Build, Rocket, State};
use rocket_db_pools::Connection;
use rocket_db_pools::Database;
use sha256::digest;
use sqlx::PgConnection;
use std::env;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use text_splitter::TextSplitter;
use tools::{AddTool, Content, TimeTool, Tool};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing::{Instrument, Level, span};
use tracing_subscriber::{Registry, prelude::*};

#[derive(Database)]
#[database("draid")]
struct DBDraid(rocket_db_pools::sqlx::PgPool);

#[derive(Database)]
#[database("kb")]
struct DBKb(rocket_db_pools::sqlx::PgPool);

async fn run_migrations_draid(rocket: Rocket<Build>) -> fairing::Result {
    match DBDraid::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => {
                let password = env::var("INIT_ADMIN_PASSWORD").unwrap();
                let admin_user = UserRequest {
                    username: "admin",
                    password: Some(&password), //intentinoally don't start up if not set
                    roles: vec![Role::Admin],
                };
                let mut connection = db.0.acquire().await.unwrap();
                if psql_users::get_user(&admin_user.username, &mut *connection)
                    .await
                    .is_err()
                {
                    create_user(&admin_user, &mut *connection).await.unwrap();
                };
                Ok(rocket)
            }
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}
async fn run_migrations_kb(rocket: Rocket<Build>) -> fairing::Result {
    match DBKb::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations-vector").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

async fn create_logging(rocket: Rocket<Build>) -> fairing::Result {
    match DBDraid::fetch(&rocket) {
        Some(db) => {
            let (tx, rx) = mpsc::channel(1);

            // Spawn the worker task onto the tokio runtime
            let worker_handle = rocket::tokio::spawn(run_async_worker(AsyncDbWorker {
                rx,
                db_client: db.0.clone(),
            }));

            let layer = PSqlLayer {
                tx: Arc::new(Mutex::new(tx)),
            };

            // Optional: Add an EnvFilter layer for runtime filtering
            let filter_layer = tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();

            let subscriber = Registry::default()
                .with(filter_layer) // Handles RUST_LOG environment variable filtering
                .with(layer); // Your custom processing layer

            tracing::subscriber::set_global_default(subscriber)
                .expect("Failed to set global tracing subscriber");

            Ok(rocket.manage(worker_handle))
        }
        None => Err(rocket),
    }
}

fn generate_bots(
    model_name: String,
    open_ai_compatable_endpoint: String,
) -> impl FnOnce(Rocket<Build>) -> Pin<Box<dyn Future<Output = fairing::Result> + Send>> {
    move |rocket: Rocket<Build>| {
        Box::pin(async move {
            match DBKb::fetch(&rocket) {
                Some(db) => {
                    let mut connection = db.0.acquire().await.unwrap();
                    let kb_arcs: Vec<Arc<dyn Tool + Send + Sync>> =
                        vec![kb!("recipes", 3), kb!("gardening", 3)];
                    for kb_arc in kb_arcs.iter() {
                        match write_knowledge_base(kb_arc.name(), &mut *connection).await {
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
                    Ok(rocket.manage(bots))
                }
                None => Err(rocket),
            }
        })
    }
}

struct Bots {
    helper_bot: Bot,
    tutor_bot: Bot,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let open_ai_compatable_endpoint = env::var("OPEN_AI_COMPATABLE_ENDPOINT")
        .unwrap_or_else(|_e| "http://localhost:11434".to_string());

    let jwt_secret = env::var("JWT_SECRET").unwrap().into_bytes();
    //Temperature=0.6, TopP=0.95, TopK=20, and MinP=0, PresencePenalty=1.5
    let model_name = "hf.co/Qwen/Qwen3-4B-GGUF:latest";

    let embedding_client = Arc::new(EmbeddingClient::new(
        //"bge-m3:567m".to_string(),
        "hf.co/mixedbread-ai/mxbai-embed-large-v1".to_string(),
        &open_ai_compatable_endpoint,
    ));

    let rocket = rocket::build()
        .attach(DBDraid::init())
        .attach(DBKb::init())
        .attach(AdHoc::try_on_ignite(
            "DBDraid Migrations",
            run_migrations_draid,
        ))
        .attach(AdHoc::try_on_ignite("DBKb Migrations", run_migrations_kb))
        .attach(AdHoc::try_on_ignite("Logging", create_logging))
        .attach(AdHoc::try_on_ignite(
            "Bots",
            generate_bots(open_ai_compatable_endpoint, model_name.to_string()),
        ))
        //.manage(bots)
        .manage(jwt_secret)
        .manage(embedding_client)
        .mount(
            "/",
            routes![
                helper,
                tutor,
                new_user,
                get_users,
                get_user,
                delete_user,
                update_user,
                login,
                new_session,
                delete_session,
                latest_session,
                get_sessions,
                get_messages,
                tool_use,
                histogram,
                create_kb,
                similar_kb_by_id,
                similar_kb_by_name,
                get_kbs,
                ingest_kb_by_id,
                ingest_kb_by_name
            ],
        )
        .ignite()
        .await?;

    rocket.launch().await?;
    Ok(())
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Prompt<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct PromptKb<'a> {
    text: &'a str,
    num_results: i16,
}

async fn chat_with_bot(
    bot: Bot,
    psql_memory: PsqlMemory,
    prompt: String,
) -> Result<TextStream![String], BadRequest<String>> {
    let messages = psql_memory
        .messages()
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    let tx_persist_message = manage_chat_interaction(&prompt, psql_memory)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    let (tx, mut rx) = mpsc::channel::<String>(1);

    let span_id = Uuid::new_v4().to_string();
    //frustrating that I'm cloning...I tried to get bot and prompt to be efficient
    rocket::tokio::spawn(async move {
        if let Err(e) = chat_with_tools(bot, tx, &messages, prompt, span_id)
            .instrument(span!(Level::INFO, "chat_with_tools", tool_use = false))
            .await
        {
            eprintln!("chat_with_tools exploded: {}", e); // Or propagate if you care
        }
    });
    Ok(TextStream! {
        while let Some(chunk) = rx.recv().await {
            if let Err(e) = tx_persist_message.send(chunk.clone()).await {
                eprintln!("Failed to send chunk to background task: {}", e);
            }
            yield chunk
        }
    })
}

#[derive(Debug, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
struct AuthRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct AuthResponse {
    access_token: String,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
enum ResponseStatus {
    Success,
    //Failure,
}
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct StatusResponse {
    status: ResponseStatus,
}

#[post("/user", format = "json", data = "<user>")]
async fn new_user<'a>(
    user: Json<psql_users::UserRequest<'a>>,
    mut db: Connection<DBDraid>,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::create_user(&user, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[delete("/user/<id>")]
async fn delete_user<'a>(
    id: Uuid,
    mut db: Connection<DBDraid>,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::delete_user(&id, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[patch("/user/<id>", format = "json", data = "<user>")]
async fn update_user<'a>(
    id: Uuid,
    user: Json<psql_users::UserRequest<'a>>,
    mut db: Connection<DBDraid>,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::patch_user(&id, &user, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[get("/user")]
async fn get_users<'a>(
    mut db: Connection<DBDraid>,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<Vec<UserResponse>>, BadRequest<String>> {
    let users = psql_users::get_all_users(&mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(users))
}

#[get("/user/me")]
async fn get_user<'a>(
    mut db: Connection<DBDraid>,
    user: auth::AuthenticatedUser,
) -> Result<Json<UserResponse>, BadRequest<String>> {
    let user = psql_users::get_user(&user.username, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(user))
}

#[post("/session", format = "json")]
async fn new_session<'a>(
    mut db: Connection<DBDraid>,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<SessionDB>, BadRequest<String>> {
    let session = psql_users::create_session(&user.id, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(session))
}

#[delete("/session/<session_id>")]
async fn delete_session(
    session_id: Uuid,
    mut db: Connection<DBDraid>,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::delete_session(&session_id, &user.id, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[get("/session", format = "json")]
async fn get_sessions<'a>(
    mut db: Connection<DBDraid>,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<Vec<SessionDB>>, BadRequest<String>> {
    let sessions = psql_users::get_all_sessions(&user.id, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(sessions))
}

#[get("/session/recent", format = "json")]
async fn latest_session<'a>(
    mut db: Connection<DBDraid>,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<Option<SessionDB>>, BadRequest<String>> {
    let session = psql_users::get_most_recent_session(&user.id, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(session))
}

#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Form<AuthRequest<'_>>,
    mut db: Connection<DBDraid>,
    jwt_secret: &State<Vec<u8>>,
) -> Result<Json<AuthResponse>, Status> {
    psql_users::authenticate_user(&credentials.username, &credentials.password, &mut db)
        .await
        .map_err(|_e| Status::Unauthorized)?;

    let access_token = auth::create_token(credentials.username.to_string(), &jwt_secret)
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AuthResponse { access_token }))
}

#[get("/messages/<session_id>")]
async fn get_messages(
    session_id: Uuid,
    db: &DBDraid,
    user: auth::AuthenticatedUser,
) -> Result<Json<Vec<MessageResult>>, BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, user.id, db.0.clone());
    let messages = psql_memory
        .messages()
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(messages))
}

#[post("/helper?<session_id>", format = "json", data = "<prompt>")]
async fn helper<'a>(
    session_id: Uuid,
    prompt: Json<Prompt<'a>>,
    db: &DBDraid,
    bots: &State<Bots>,
    helper: auth::Helper,
) -> Result<TextStream![String], BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, helper.id, db.0.clone());
    chat_with_bot(
        bots.helper_bot.clone(),
        psql_memory,
        prompt.text.to_string(),
    )
    .await
}

#[post("/tutor?<session_id>", format = "json", data = "<prompt>")]
async fn tutor<'a>(
    session_id: Uuid,
    prompt: Json<Prompt<'a>>,
    db: &DBDraid,
    bots: &State<Bots>,
    tutor: auth::Tutor,
) -> Result<TextStream![String], BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, tutor.id, db.0.clone());
    chat_with_bot(bots.tutor_bot.clone(), psql_memory, prompt.text.to_string()).await
}

#[get("/telemetry/latency/<endpoint>", format = "json")]
async fn histogram(
    endpoint: &str,
    mut db: Connection<DBDraid>,
    _admin: auth::Admin,
) -> Result<Json<Vec<HistogramIncrement>>, BadRequest<String>> {
    let results = get_histogram(&mut db, &endpoint)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(results))
}

#[get("/span/tools/<endpoint>", format = "json")]
async fn tool_use(
    endpoint: &str,
    mut db: Connection<DBDraid>,
    _admin: auth::Admin,
) -> Result<Json<Vec<SpanToolUse>>, BadRequest<String>> {
    let results = get_tool_use(&mut db, &endpoint)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(results))
}

async fn similar_content<'a>(
    kb_id: i64,
    prompt: Json<PromptKb<'a>>,
    mut db: Connection<DBKb>,
    client: &EmbeddingClient,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    let embeddings = get_embeddings(&client, &prompt.text)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    let result = get_similar_content(kb_id, embeddings, prompt.num_results, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(result))
}

#[post("/knowledge_base/<kb_id>/similar", format = "json", data = "<prompt>")]
async fn similar_kb_by_id<'a>(
    kb_id: i64,
    prompt: Json<PromptKb<'a>>,
    db: Connection<DBKb>,
    client: &State<Arc<EmbeddingClient>>,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    similar_content(kb_id, prompt, db, client).await
}

#[post(
    "/knowledge_base/<kb>/similar",
    format = "json",
    data = "<prompt>",
    rank = 2
)]
async fn similar_kb_by_name<'a>(
    kb: &str,
    prompt: Json<PromptKb<'a>>,
    mut db: Connection<DBKb>,
    client: &State<Arc<EmbeddingClient>>,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    let KnowledgeBase { id, .. } = get_knowledge_base(kb, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    similar_content(id, prompt, db, client).await
}
async fn extract_and_write(
    client: &EmbeddingClient,
    document_id: i64,
    kb_id: i64,
    chunk: String,
    mut db: &mut PgConnection,
) -> anyhow::Result<()> {
    let embeddings = get_embeddings(&client, &chunk).await?;
    write_single_content(document_id, kb_id, &chunk, embeddings, &mut db).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct KBRequest<'a> {
    name: &'a str,
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct KBResponse {
    id: i64,
}

//curl --header "Content-Type: application/json" -X POST http://127.0.0.1:8001/knowledge_base --data '{"name": "paul_graham"}'
#[post("/knowledge_base", format = "json", data = "<data>")]
async fn create_kb<'a>(
    data: Json<KBRequest<'a>>,
    mut db: Connection<DBKb>,
) -> Result<Json<KBResponse>, BadRequest<String>> {
    let id = write_knowledge_base(&data.name, &mut db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(KBResponse { id }))
}

#[get("/knowledge_base")]
async fn get_kbs(mut db: Connection<DBKb>) -> Result<Json<Vec<KnowledgeBase>>, BadRequest<String>> {
    Ok(Json(
        get_knowledge_bases(&mut db)
            .await
            .map_err(|e| BadRequest(e.to_string()))?,
    ))
}

async fn ingest_content(
    kb_id: i64, //category of knowledge base
    data: Data<'_>,
    db: &DBKb,
    client: &EmbeddingClient,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    let max_characters = 1000;
    let splitter = TextSplitter::new(max_characters);
    let span_id = Uuid::new_v4().to_string();
    info!(
        tool_use = true,
        is_kb = true,
        span_id,
        "Started ingesting content"
    );
    let content = data
        .open(20.mebibytes())
        .into_string()
        .await
        .map_err(|e| BadRequest(e.to_string()))?
        .into_inner();
    let content_hash = digest(&content);
    let mut conn = db.acquire().await.map_err(|e| BadRequest(e.to_string()))?;
    match write_document(&content_hash, &mut conn).await {
        Ok(document_id) => {
            info!(tool_use = true, span_id, "Finished reading content");
            let chunks: Vec<String> = splitter.chunks(&content).map(|v| v.to_string()).collect();
            info!(
                tool_use = true,
                is_kb = true,
                span_id,
                "Finished chunking content"
            );
            info!(
                tool_use = true,
                is_kb = true,
                span_id,
                message = format!("Number of chunks {}", chunks.len())
            );
            let futures = chunks.into_iter().map(|chunk| async move {
                let mut conn = db.acquire().await?;
                extract_and_write(&client, document_id, kb_id, chunk, &mut conn).await
            });
            let results: Vec<anyhow::Result<()>> = stream::iter(futures)
                .buffer_unordered(100) // Concurrently process up to 100 tasks
                .collect()
                .await;
            info!(
                tool_use = true,
                is_kb = true,
                span_id,
                "Finished writing vectors"
            );
            for result in results {
                result.map_err(|e| BadRequest(e.to_string()))?;
            }
        }
        Err(_e) => {
            info!(tool_use = true, is_kb = true, span_id, "Already indexed!");
        }
    }
    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

//curl --header "Content-Type: application/json"  -X POST http://127.0.0.1:8001/content/similar --data '{"text": "what did paul graham primary work on?", "num_results": 3}'
//curl -X POST http://127.0.0.1:8001/knowledge_base/1/ingest --data '@paul_graham_essay.txt'
#[post("/knowledge_base/<kb_id>/ingest", data = "<data>")]
async fn ingest_kb_by_id(
    kb_id: i64, //category of knowledge base
    data: Data<'_>,
    db: &DBKb,
    client: &State<Arc<EmbeddingClient>>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    ingest_content(kb_id, data, db, client)
        .instrument(span!(
            Level::INFO,
            "knowledge_base_ingest",
            tool_use = false
        ))
        .await
}

#[post("/knowledge_base/<kb>/ingest", data = "<data>", rank = 2)]
async fn ingest_kb_by_name(
    kb: &str, //category of knowledge base
    data: Data<'_>,
    db: &DBKb,
    client: &State<Arc<EmbeddingClient>>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    let mut conn = db.acquire().await.map_err(|e| BadRequest(e.to_string()))?;
    let KnowledgeBase { id, .. } = get_knowledge_base(kb, &mut conn)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    ingest_content(id, data, db, client)
        .instrument(span!(
            Level::INFO,
            "knowledge_base_ingest",
            tool_use = false
        ))
        .await
}
