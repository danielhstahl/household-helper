#[macro_use]
extern crate rocket;

mod llm;
use llm::{EmbeddingClient, get_embedding_client, get_embeddings, get_embeddings_batch};
mod psql_vectors;
use anyhow;
use futures::stream::{self, StreamExt};
use psql_vectors::{SimilarContent, get_similar_content, write_content, write_single_content};
use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{self, AdHoc};
use rocket::fs::TempFile;
use rocket::response::status::BadRequest;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::tokio::io::AsyncReadExt;
use rocket::{Build, Rocket, State};
use rocket_db_pools::Database;
use sqlx::{Error, Pool, Postgres, Row, postgres::PgRow};
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use text_splitter::TextSplitter;

#[derive(Database)]
#[database("draid")]
struct Db(rocket_db_pools::sqlx::PgPool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
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
        .mount("/", routes![ingest_content, similar_content])
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

#[post("/content/similar", format = "json", data = "<prompt>")]
async fn similar_content<'a>(
    prompt: Json<Prompt<'a>>,
    db: &Db,
    client: &State<EmbeddingClient>,
) -> Result<Json<Vec<SimilarContent>>, BadRequest<String>> {
    let embeddings = get_embeddings(&client, &prompt.text)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    let result = get_similar_content(embeddings, prompt.num_results, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(result))
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
    chunk: String,
    db: sqlx::Pool<Postgres>,
) -> anyhow::Result<()> {
    //need to clone, behind the scenes the Ollama library was making a copy anyway
    let embeddings = get_embeddings(&client, &chunk).await?;
    //let embeddings = embeddings.pop().unwrap();
    write_single_content(&chunk, embeddings, &db).await?;
    Ok(())
}

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
    let futures = chunks
        .into_iter()
        .map(|chunk| extract_and_write(client.inner().clone(), chunk, db.0.clone()));
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
    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

//
/*let mut stream = data.open(20.mebibytes());
let mut content = String::new();
// Loop to read the file in chunks.
while let Ok(n) = stream.read(&mut buffer).await {
    // If n is 0, the stream has ended.
    if n == 0 {
        break;
    }
    let large_chunk = match str::from_utf8(&buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    content.push_str(large_chunk);
}*/
