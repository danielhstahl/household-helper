use crate::llm::Bot;
use crate::prompts::{HELPER_PROMPT, TUTOR_PROMPT};
use crate::psql_memory::MessageResult;
use crate::psql_users::{SessionDB, UserResponse};
use crate::tools::Tool;
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, Enum, Object};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Object)]
pub struct AuthResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize, Object)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

pub struct Bots {
    pub helper_bot: Arc<Bot>,
    pub tutor_bot: Arc<Bot>,
}

pub fn get_bots(
    model_name: String,
    open_ai_compatable_endpoint: String,
    helper_tools: Vec<Arc<dyn Tool + Send + Sync>>,
) -> Bots {
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
    bots
}

#[derive(Debug)]
pub struct NoData {
    pub msg: String,
}
impl fmt::Display for NoData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for NoData {}

#[derive(Debug)]
pub struct LLMError {
    pub msg: String,
}
impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for LLMError {}

#[derive(Debug, Serialize, Enum)]
pub enum ResponseStatus {
    Success,
    //Failure,
}

#[derive(Debug, Serialize, Object)]
pub struct StatusResponse {
    pub status: ResponseStatus,
}

#[derive(ApiResponse)]
pub enum SuccessResponse {
    // Status 200: Success
    #[oai(status = 200)]
    Success(Json<StatusResponse>),
}

#[derive(ApiResponse)]
pub enum UsersResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<UserResponse>>),

    #[oai(status = 200)]
    SuccessSingle(Json<UserResponse>),
}

#[derive(ApiResponse)]
pub enum SessionResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<SessionDB>>),

    #[oai(status = 200)]
    SuccessSingle(Json<SessionDB>),
}

#[derive(ApiResponse)]
pub enum MessageResponse {
    // Status 200: Success
    #[oai(status = 200)]
    SuccessMultiple(Json<Vec<MessageResult>>),
}

#[derive(Deserialize)] // No deny_unknown_fields; let token slide, it's inert here
pub struct SessionQuery {
    pub session_id: Uuid,
}
#[derive(Object, Serialize)]
pub struct UploadResponse {
    pub filename: String,
    pub size: usize,
    pub hash: String, // e.g., blake3 for that cyberpunk veracity
}

#[derive(Deserialize, Object)]
pub struct PromptKb {
    pub text: String,
    pub num_results: i16,
}
