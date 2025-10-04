#[macro_use]
extern crate rocket;

mod llm;
mod psql_vectors;

use anyhow;
use futures::stream::{self, StreamExt};
use llm::{EmbeddingClient, get_embedding_client, get_embeddings};
use psql_vectors::{
    KnowledgeBase, SimilarContent, get_knowledge_bases, get_similar_content, write_document,
    write_knowledge_base, write_single_content,
};
use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{self, AdHoc};
use rocket::response::status::BadRequest;
use rocket::serde::{Deserialize, Serialize, json, json::Json};
use rocket::{Build, Rocket, State};
use rocket_db_pools::Database;
use sha256::digest;
use sqlx::Postgres;
use std::env;
use std::time::Instant;
use text_splitter::TextSplitter;

use crate::psql_vectors::get_knowledge_base;

#[derive(Database)]
#[database("kb")]
struct Db(rocket_db_pools::sqlx::PgPool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => {
                let raw_kb_names = match env::var("KNOWLEDGE_BASE_NAMES") {
                    Ok(v) => v,
                    Err(_e) => "[]".to_string(),
                };
                let knowledge_base_names: Vec<&str> =
                    json::from_str(raw_kb_names.as_str()).unwrap();
                for name in knowledge_base_names {
                    match write_knowledge_base(name, &**db).await {
                        Ok(v) => println!("Created knowledge base {} with index {}", name, v),
                        Err(e) => println!("Failed to create knowledge base: {}", e),
                    }
                }
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

struct AiConfig {
    ollama_endpoint: String,
}

#[launch]
fn rocket() -> _ {
    let ai_config = AiConfig {
        ollama_endpoint: match env::var("OLLAMA_ENDPOINT") {
            Ok(v) => v,
            Err(_e) => "http://localhost:11434".to_string(),
        },
    };

    let embedding_client =
        get_embedding_client(&ai_config.ollama_endpoint, "bge-m3:567m".to_string());

    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .manage(ai_config)
        .manage(embedding_client)
        .mount(
            "/",
            routes![
                similar_kb_by_id,
                similar_kb_by_name,
                create_kb,
                get_kbs,
                ingest_kb_by_id,
                ingest_kb_by_name
            ],
        )
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Prompt<'a> {
    text: &'a str,
    num_results: i16,
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

async fn similar_content<'a>(
    kb_id: i64,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    let embeddings = get_embeddings(&client, &prompt.text)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    let result = get_similar_content(kb_id, embeddings, prompt.num_results, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(result))
}

#[post("/knowledge_base/<kb_id>/similar", format = "json", data = "<prompt>")]
async fn similar_kb_by_id<'a>(
    kb_id: i64,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    client: &State<EmbeddingClient>,
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
    prompt: Json<Prompt<'a>>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    let KnowledgeBase { id, .. } = get_knowledge_base(kb, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    similar_content(id, prompt, db, client).await
}
//oddly, this is slower (!) than the individual writing approach
/*
#[post("/content/ingest", data = "<data>")]
async fn ingest_content(
    data: Data<'_>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    let max_characters = 1000;
    let splitter = TextSplitter::new(max_characters);
    println!("started reading content");
    let content = data
        .open(20.mebibytes())
        .into_string()
        .await
        .map_err(|e| BadRequest(e.to_string()))?
        .into_inner();
    println!("finished reading content");
    let chunks: Vec<String> = splitter.chunks(&content).map(|v| v.to_string()).collect();
    println!("finished chunking content");
    println!("num chunks: {}", chunks.len());
    let start = Instant::now();

    // Wrap client and db in Arc for shared ownership across tasks
    let client = Arc::new(client.inner().clone());
    let db = Arc::new(db.0.clone());

    let chunked: Vec<Vec<String>> = chunks.chunks(10).map(|chunk| chunk.to_vec()).collect();

    let futures = chunked.into_iter().map(|chunk| {
        let client = Arc::clone(&client);
        let db = Arc::clone(&db);
        //let chunk = chunk.to_vec();
        async move { extract_and_write(client, chunk, db).await }
    });

    let results: Vec<anyhow::Result<()>> =
        stream::iter(futures).buffer_unordered(100).collect().await;

    let duration = start.elapsed();
    println!("Time elapsed in writing to vector is: {:?}", duration);
    println!("finished writing vectors");
    for result in results {
        result.map_err(|e| BadRequest(e.to_string()))?;
    }
    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

async fn extract_and_write(
    client: Arc<EmbeddingClient>,
    chunk: Vec<String>,
    db: Arc<sqlx::Pool<Postgres>>,
) -> anyhow::Result<()> {
    let embeddings = get_embeddings_batch(&client, chunk.clone()).await?;
    write_content(chunk, embeddings, &db).await?;
    Ok(())
}
*/

async fn extract_and_write(
    client: EmbeddingClient,
    document_id: i64,
    kb_id: i64,
    chunk: String,
    db: sqlx::Pool<Postgres>,
) -> anyhow::Result<()> {
    let embeddings = get_embeddings(&client, &chunk).await?;
    write_single_content(document_id, kb_id, &chunk, embeddings, &db).await?;
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
    db: &Db,
) -> Result<Json<KBResponse>, BadRequest<String>> {
    let id = write_knowledge_base(&data.name, &db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(KBResponse { id }))
}

#[get("/knowledge_base")]
async fn get_kbs(db: &Db) -> Result<Json<Vec<KnowledgeBase>>, BadRequest<String>> {
    Ok(Json(
        get_knowledge_bases(&db)
            .await
            .map_err(|e| BadRequest(e.to_string()))?,
    ))
}

async fn ingest_content(
    kb_id: i64, //category of knowledge base
    data: Data<'_>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    let max_characters = 1000;
    let splitter = TextSplitter::new(max_characters);
    println!("started reading content");
    let content = data
        .open(20.mebibytes())
        .into_string()
        .await
        .map_err(|e| BadRequest(e.to_string()))?
        .into_inner();
    let content_hash = digest(&content);
    match write_document(&content_hash, &db.0).await {
        Ok(document_id) => {
            println!("finished reading content");
            let chunks: Vec<String> = splitter.chunks(&content).map(|v| v.to_string()).collect();
            println!("finished chunking content");
            println!("num chunks: {}", chunks.len());
            let start = Instant::now();
            let futures = chunks.into_iter().map(|chunk| {
                extract_and_write(
                    client.inner().clone(),
                    document_id,
                    kb_id,
                    chunk,
                    db.0.clone(),
                )
            });
            let results: Vec<anyhow::Result<()>> = stream::iter(futures)
                .buffer_unordered(100) // Concurrently process up to 100 tasks
                .collect()
                .await;
            let duration = start.elapsed();
            println!("Time elapsed in writing to vector is: {:?}", duration);
            println!("finished writing vectors");
            for result in results {
                result.map_err(|e| BadRequest(e.to_string()))?;
            }
        }
        Err(_e) => {
            println!("Already indexed!");
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
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    ingest_content(kb_id, data, db, client).await
}

#[post("/knowledge_base/<kb>/ingest", data = "<data>", rank = 2)]
async fn ingest_kb_by_name(
    kb: &str, //category of knowledge base
    data: Data<'_>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    let KnowledgeBase { id, .. } = get_knowledge_base(kb, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    ingest_content(id, data, db, client).await
}
