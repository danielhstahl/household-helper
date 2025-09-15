#[macro_use]
extern crate rocket;
mod auth;
mod llm;
mod psql_users;
use jsonwebtoken::{EncodingKey, Header, encode};
use llm::{chat, get_bot, get_llm};
mod prompts;
mod psql_memory;
use async_openai::{Client, config::OpenAIConfig, types::CreateChatCompletionRequest};
use futures::StreamExt;
use prompts::HELPER_PROMPT;
use psql_memory::PsqlMemory;
use psql_memory::manage_chat_interaction;
use rocket::fairing::{self, AdHoc};
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::response::stream::TextStream;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{Build, Rocket, State, futures};
use rocket_db_pools::Database;
use sqlx::types::Uuid;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

//TODO, pass JWT_SECRET as env variable
use crate::auth::{Claims, JWT_SECRET};
use crate::prompts::TUTOR_PROMPT;

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
    lm_studio_endpoint: String,
    ollama_endpoint: String,
}

struct Bots {
    helper_bot: CreateChatCompletionRequest,
    tutor_bot: CreateChatCompletionRequest,
}

#[launch]
fn rocket() -> _ {
    let ai_config = AiConfig {
        lm_studio_endpoint: match env::var("LM_STUDIO_ENDPOINT") {
            Ok(v) => v,
            Err(_e) => "http://localhost:1234".to_string(),
        },
        ollama_endpoint: match env::var("OLLAMA_ENDPOINT") {
            Ok(v) => v,
            Err(_e) => "http://localhost:11434".to_string(),
        },
    };

    let model_name = "qwen3-8b";
    //happens at startup, so can "safely" unwrap

    let bots = Bots {
        helper_bot: get_bot(&model_name, &HELPER_PROMPT).unwrap(),
        tutor_bot: get_bot(&model_name, &TUTOR_PROMPT).unwrap(),
    };

    let llm = get_llm(&ai_config.lm_studio_endpoint);

    //todo, get embeddings

    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .manage(ai_config)
        .manage(llm)
        .manage(bots)
        .mount("/", routes![helper, tutor])
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Prompt<'a> {
    text: &'a str,
}

async fn chat_with_bot(
    llm: &Client<OpenAIConfig>,
    psql_memory: PsqlMemory,
    bot: CreateChatCompletionRequest,
    prompt: &str,
) -> Result<TextStream![String], BadRequest<String>> {
    let messages = psql_memory
        .messages()
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    let tx = manage_chat_interaction(&prompt, psql_memory)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    let mut stream = chat(&llm, bot, &messages, &prompt)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(TextStream! {
        while let Some(result) = stream.next().await {
            match result {
                Ok(value) =>  {
                    //typically only single item in value.choices,
                    // but if not concatenate them
                    let tokens = value.choices.iter().filter_map(|chat_choice|{
                        (&chat_choice.delta.content).as_ref()
                    }).map(|s| &**s).collect::<Vec<&str>>().join("");

                    if let Err(e) = tx.send(tokens.clone()).await {
                        eprintln!("Failed to send chunk to background task: {}", e);
                    }
                    yield tokens
                },
                Err(e) => yield e.to_string()
            }
        }
    })
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct AuthResponse {
    token: String,
}

#[post("/login", format = "json", data = "<credentials>")]
fn login(credentials: Json<AuthRequest>) -> Result<Json<AuthResponse>, Status> {
    // Hardcoded credentials for demonstration. In a real app, you'd check a database.
    if credentials.username != "admin" || credentials.password != "password123" {
        return Err(Status::Unauthorized);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        sub: credentials.username.clone(),
        iat: now,
        exp: now + (60 * 30), // 30 minutes expiration
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AuthResponse { token }))
}
/**
* Example invocation:
* curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"text":"hello world!"}' \
  'http://127.0.0.1:8000/helper?session_id=9e186ac2-3912-4250-83d2-d5b779729f52'
*/
#[post("/helper?<session_id>", format = "json", data = "<prompt>")]
async fn helper<'a>(
    session_id: &str,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    llm: &State<Client<OpenAIConfig>>,
    bots: &State<Bots>,
) -> Result<TextStream![String], BadRequest<String>> {
    let session_id = Uuid::parse_str(session_id).map_err(|e| BadRequest(e.to_string()))?;
    let psql_memory = PsqlMemory::new(100, session_id, db.0.clone());
    chat_with_bot(&llm, psql_memory, bots.helper_bot.clone(), &prompt.text).await
}

#[post("/tutor?<session_id>", format = "json", data = "<prompt>")]
async fn tutor<'a>(
    session_id: &str,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    llm: &State<Client<OpenAIConfig>>,
    bots: &State<Bots>,
) -> Result<TextStream![String], BadRequest<String>> {
    let session_id = Uuid::parse_str(session_id).map_err(|e| BadRequest(e.to_string()))?;
    let psql_memory = PsqlMemory::new(100, session_id, db.0.clone());
    chat_with_bot(&llm, psql_memory, bots.tutor_bot.clone(), &prompt.text).await
}

//#[get("/session")]
