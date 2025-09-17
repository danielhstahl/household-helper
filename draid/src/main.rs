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
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::response::stream::TextStream;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{Build, Rocket, State, futures};
use rocket_db_pools::Database;
//use sqlx::types::Uuid;
use rocket::serde::uuid::Uuid;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

//TODO, pass JWT_SECRET as env variable
use auth::{Claims, JWT_SECRET};
use prompts::TUTOR_PROMPT;
use psql_memory::Message;
use psql_users::{Role, SessionDB, UserRequest, UserResponse, create_user};

#[derive(Database)]
#[database("draid")]
struct Db(rocket_db_pools::sqlx::PgPool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => {
                let password = env::var("INIT_ADMIN_PASSWORD").unwrap();
                let admin_user = UserRequest {
                    username: "admin",
                    password: &password, //intentinoally don't start up if not set
                    roles: vec![Role::Admin],
                };
                if psql_users::get_user(&admin_user.username, &**db)
                    .await
                    .is_err()
                {
                    create_user(&admin_user, &**db).await.unwrap();
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
                get_messages
            ],
        )
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

#[derive(Debug, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
struct AuthRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct AuthResponse {
    token: String,
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
    db: &Db,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::create_user(&user, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[delete("/user/<id>")]
async fn delete_user<'a>(
    id: Uuid,
    db: &Db,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::delete_user(&id, &db.0)
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
    db: &Db,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::patch_user(&id, &user, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[get("/user")]
async fn get_users<'a>(
    db: &Db,
    _admin: auth::Admin, //guard, only admins can access this
) -> Result<Json<Vec<UserResponse>>, BadRequest<String>> {
    let users = psql_users::get_all_users(&db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(users))
}

#[get("/user/me")]
async fn get_user<'a>(
    db: &Db,
    user: auth::AuthenticatedUser,
) -> Result<Json<UserResponse>, BadRequest<String>> {
    let user = psql_users::get_user(&user.username, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(user))
}

#[post("/session", format = "json")]
async fn new_session<'a>(
    db: &Db,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::create_session(&user.id, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[delete("/session/<session_id>")]
async fn delete_session(
    session_id: Uuid,
    db: &Db,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<StatusResponse>, BadRequest<String>> {
    psql_users::delete_session(&session_id, &user.id, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(StatusResponse {
        status: ResponseStatus::Success,
    }))
}

#[get("/session", format = "json")]
async fn get_sessions<'a>(
    db: &Db,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<Vec<SessionDB>>, BadRequest<String>> {
    let sessions = psql_users::get_all_sessions(&user.id, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(sessions))
}

#[get("/session/recent", format = "json")]
async fn latest_session<'a>(
    db: &Db,
    user: auth::AuthenticatedUser, //guard, only authenticated users can access
) -> Result<Json<Option<SessionDB>>, BadRequest<String>> {
    let session = psql_users::get_most_recent_session(&user.id, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(session))
}

#[post("/login", data = "<credentials>")]
async fn login(credentials: Form<AuthRequest<'_>>, db: &Db) -> Result<Json<AuthResponse>, Status> {
    psql_users::authenticate_user(&credentials.username, &credentials.password, &db.0)
        .await
        .map_err(|_e| Status::Unauthorized)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        sub: credentials.username.to_string(),
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

#[get("/messages/<session_id>")]
async fn get_messages(
    session_id: Uuid,
    db: &Db,
    user: auth::AuthenticatedUser,
) -> Result<Json<Vec<Message>>, BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, user.id, db.0.clone());
    let messages = psql_memory
        .messages()
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(messages))
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
    session_id: Uuid,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    llm: &State<Client<OpenAIConfig>>,
    bots: &State<Bots>,
    helper: auth::Helper,
) -> Result<TextStream![String], BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, helper.id, db.0.clone());
    chat_with_bot(&llm, psql_memory, bots.helper_bot.clone(), &prompt.text).await
}

#[post("/tutor?<session_id>", format = "json", data = "<prompt>")]
async fn tutor<'a>(
    session_id: Uuid,
    prompt: Json<Prompt<'a>>,
    db: &Db,
    llm: &State<Client<OpenAIConfig>>,
    bots: &State<Bots>,
    tutor: auth::Tutor,
) -> Result<TextStream![String], BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, tutor.id, db.0.clone());
    chat_with_bot(&llm, psql_memory, bots.tutor_bot.clone(), &prompt.text).await
}
