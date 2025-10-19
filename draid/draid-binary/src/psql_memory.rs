use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use sqlx::{Pool, Postgres, Type, types::Uuid};
#[derive(Serialize, Deserialize, Type, Enum)]
#[sqlx(type_name = "message_type")]
pub enum MessageType {
    #[serde(rename = "system")]
    #[sqlx(rename = "system")]
    SystemMessage,
    #[serde(rename = "ai")]
    #[sqlx(rename = "ai")]
    AIMessage,
    #[serde(rename = "human")]
    #[sqlx(rename = "human")]
    HumanMessage,
    #[serde(rename = "tool")]
    #[sqlx(rename = "tool")]
    ToolMessage,
}

#[derive(Serialize, sqlx::FromRow, Object)]
pub struct MessageResult {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub message_type: MessageType,
}

pub struct PsqlMemory {
    num_messages: usize,
    session_id: Uuid,
    username_id: Uuid,
    pool: Pool<Postgres>,
}

impl PsqlMemory {
    pub fn new(
        num_messages: usize,
        session_id: Uuid,
        username_id: Uuid,
        pool: Pool<Postgres>,
    ) -> Self {
        Self {
            num_messages,
            session_id,
            username_id,
            pool,
        }
    }
    pub async fn messages(&self) -> sqlx::Result<Vec<MessageResult>> {
        sqlx::query_as!(
            MessageResult,
            r#"
            SELECT content as "content: String",
            message_type as "message_type: MessageType",
            message_ts as "timestamp"
            FROM messages WHERE session_id = $1
            AND username_id = $2
            ORDER BY message_ts limit $3
            "#,
            &self.session_id,
            &self.username_id,
            self.num_messages as i32
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_message(&self, message: Message) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO messages (id, content, message_type, session_id, username_id, message_ts)
            VALUES(gen_random_uuid(), $1, $2, $3, $4, NOW())
            "#,
            &message.content,
            message.message_type as MessageType,
            &self.session_id,
            &self.username_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub async fn write_human_message(new_message: String, memory: &PsqlMemory) -> sqlx::Result<()> {
    let message = Message {
        content: new_message,
        message_type: MessageType::HumanMessage,
    };
    memory.add_message(message).await
}

pub async fn write_ai_message(new_message: String, memory: &PsqlMemory) -> sqlx::Result<()> {
    let message = Message {
        content: new_message,
        message_type: MessageType::AIMessage,
    };
    memory.add_message(message).await
}
