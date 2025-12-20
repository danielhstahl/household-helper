use futures::{FutureExt, SinkExt, StreamExt, future::BoxFuture};
use poem::{
    Error, IntoResponse, Result, handler,
    http::StatusCode,
    web::websocket::{Message, WebSocket, WebSocketStream},
    web::{Data, Form, Multipart, Path, Query as WsQuery},
};
use poem_openapi::{OpenApi, payload::Json};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{Instrument, Level, info, span};
use uuid::Uuid;

use crate::auth::{UserIdentification, create_token};
use crate::dbtracing::{HistogramIncrement, SpanToolUse, get_histogram, get_tool_use};
use crate::embedding::{EmbeddingClient, get_embeddings, ingest_content};
use crate::llm::{Bot, chat_with_tools};
use crate::models::{
    AuthRequest, AuthResponse, Bots, LLMError, MessageResponse, NoData, PromptKb, ResponseStatus,
    SessionQuery, SessionResponse, StatusResponse, SuccessResponse, UploadResponse, UsersResponse,
};
use crate::psql_memory::{PsqlMemory, write_ai_message, write_human_message};
use crate::psql_users;
use crate::psql_users::Role;
use crate::psql_vectors::{
    KnowledgeBase, get_docs_with_similar_content, get_knowledge_base, get_knowledge_bases,
};
use poem::error::InternalServerError;

fn handle_chat_session(
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
                let full_message = chat_with_tools(&bot, &mut socket, &messages, &prompt, &span_id)
                    .instrument(span!(
                        Level::INFO,
                        "chat_with_tools",
                        endpoint = "query",
                        tool_use = false
                    ))
                    .await
                    .map_err(|e| {
                        let e_str = e.to_string();
                        info!(
                            tool_use = false,
                            endpoint = "query",
                            span_id,
                            message = &e_str
                        );
                        InternalServerError(LLMError { msg: e_str })
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

#[poem_grants::protect("Role::Tutor", ty = "crate::psql_users::Role")]
#[handler]
pub async fn tutor_ws_handler(
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

#[poem_grants::protect("Role::Helper", ty = "crate::psql_users::Role")]
#[handler]
pub async fn helper_ws_handler(
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

pub struct Api;
#[poem_grants::open_api]
#[OpenApi]
impl Api {
    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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
    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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

    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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

    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
    #[oai(path = "/user", method = "get")]
    async fn get_users(&self, Data(pool): Data<&PgPool>) -> Result<UsersResponse> {
        let users = psql_users::get_all_users(pool)
            .await
            .map_err(InternalServerError)?;
        Ok(UsersResponse::SuccessMultiple(Json(users)))
    }

    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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

    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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

    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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
    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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

    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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

        let access_token = create_token(auth.username, &jwt_secret)
            .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        Ok(Json(AuthResponse { access_token }))
    }

    #[protect(
        any("Role::Admin", "Role::Tutor", "Role::Helper"),
        ty = "crate::psql_users::Role"
    )]
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

    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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
    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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
    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
    #[oai(path = "/knowledge_base", method = "get")]
    async fn get_kbs(&self, Data(pool): Data<&PgPool>) -> Result<Json<Vec<KnowledgeBase>>> {
        Ok(Json(
            get_knowledge_bases(pool)
                .await
                .map_err(InternalServerError)?,
        ))
    }

    #[protect("Role::Admin", ty = "crate::psql_users::Role")]
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
