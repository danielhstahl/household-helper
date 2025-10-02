#[macro_use]
extern crate rocket;
mod auth;
mod llm;
mod prompts;
mod psql_memory;
mod psql_users;
mod tools;

use kb_tool_macro::kb;
use llm::{Bot, chat_with_tools};
use prompts::HELPER_PROMPT;
use prompts::TUTOR_PROMPT;
use psql_memory::{MessageResult, PsqlMemory, manage_chat_interaction};
use psql_users::{Role, SessionDB, UserRequest, UserResponse, create_user};
use reqwest::Client as HttpClient;
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
use rocket_db_pools::Database;
use std::env;
use std::sync::Arc;
use tools::{AddTool, Content, Tool};

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
                    password: Some(&password), //intentinoally don't start up if not set
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
}

struct Bots {
    helper_bot: Bot,
    tutor_bot: Bot,
}

#[launch]
fn rocket() -> _ {
    let ai_config = AiConfig {
        lm_studio_endpoint: env::var("LM_STUDIO_ENDPOINT")
            .unwrap_or_else(|_e| "http://localhost:1234".to_string()),
    };
    let jwt_secret = env::var("JWT_SECRET").unwrap().into_bytes();
    let model_name = "qwen/qwen3-4b-thinking-2507";
    // the kb! macro generates code that is "impure"
    // it depends on std::env for the KB endpoint
    // The kb! calls must match what is passed to
    // knowledge-base docker at runtime.
    // see KNOWLEDGE_BASE_NAMES in docker compose
    let helper_tools: Vec<Arc<dyn Tool + Send + Sync>> =
        vec![Arc::new(AddTool), kb!("recipes", 3), kb!("gardening", 3)];

    let bots = Bots {
        helper_bot: Bot::new(
            model_name.to_string(),
            HELPER_PROMPT,
            &ai_config.lm_studio_endpoint,
            Some(helper_tools),
        ),
        tutor_bot: Bot::new(
            model_name.to_string(),
            TUTOR_PROMPT,
            &ai_config.lm_studio_endpoint,
            None,
        ),
    };

    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .manage(bots)
        .manage(jwt_secret)
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

    //frustarting...I tried to get bot and prompt to be efficient
    rocket::tokio::spawn(async move {
        if let Err(e) = chat_with_tools(bot, tx, &messages, prompt).await {
            eprintln!("chat_with_tools exploded: {}", e); // Or propagate if you care
        }
    });
    Ok(TextStream! {
        while let Some(chunk) = rx.recv().await {
            //println!("receiving final chunks");
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
) -> Result<Json<SessionDB>, BadRequest<String>> {
    let session = psql_users::create_session(&user.id, &db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(session))
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
async fn login(
    credentials: Form<AuthRequest<'_>>,
    db: &Db,
    jwt_secret: &State<Vec<u8>>,
) -> Result<Json<AuthResponse>, Status> {
    psql_users::authenticate_user(&credentials.username, &credentials.password, &db.0)
        .await
        .map_err(|_e| Status::Unauthorized)?;

    let access_token = auth::create_token(credentials.username.to_string(), &jwt_secret)
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AuthResponse { access_token }))
}

#[get("/messages/<session_id>")]
async fn get_messages(
    session_id: Uuid,
    db: &Db,
    user: auth::AuthenticatedUser,
) -> Result<Json<Vec<MessageResult>>, BadRequest<String>> {
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
    db: &Db,
    bots: &State<Bots>,
    tutor: auth::Tutor,
) -> Result<TextStream![String], BadRequest<String>> {
    let psql_memory = PsqlMemory::new(100, session_id, tutor.id, db.0.clone());
    chat_with_bot(bots.tutor_bot.clone(), psql_memory, prompt.text.to_string()).await
}
