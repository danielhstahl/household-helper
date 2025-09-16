//use langchain_rust::schemas::{ImageContent, MessageType, memory::BaseMemory, messages::Message};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::mpsc;
use rocket::tokio::sync::mpsc::Sender;
use sqlx::{Pool, Postgres, Type, types::Uuid};
#[derive(Serialize, Deserialize, Type)]
#[serde(crate = "rocket::serde")]
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
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

// TODO consider refactoring to just call functions
// rather than recreating a class and calling methods on
// the class
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
    pub async fn messages(&self) -> sqlx::Result<Vec<Message>> {
        sqlx::query_as!(
            Message,
            r#"
            SELECT content as "content: String",
            message_type as "message_type: MessageType"
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
            INSERT INTO messages (id, content, message_type, session_id, message_ts)
            VALUES(gen_random_uuid(), $1, $2, $3, NOW())
            "#,
            &message.content,
            message.message_type as MessageType,
            &self.session_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub async fn manage_chat_interaction(
    new_message: &str,
    memory: PsqlMemory,
) -> sqlx::Result<Sender<String>> {
    let (tx, mut rx) = mpsc::channel::<String>(1);
    let message = Message {
        content: new_message.to_string(),
        message_type: MessageType::HumanMessage,
    };
    memory.add_message(message).await?;
    rocket::tokio::spawn(async move {
        let mut full_response = String::new();
        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
        }

        let message = Message {
            content: full_response,
            message_type: MessageType::AIMessage,
        };
        // After the stream is complete, write to the database
        let result = memory.add_message(message).await;

        if let Err(e) = result {
            eprintln!("Failed to save message to database: {}", e);
        }
    });
    Ok(tx)
}
