use anyhow::{Ok, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPool;

#[async_trait]
pub trait Repository {
    async fn insert(&self, value: NewMessage) -> Result<()>;
    async fn find_one_by_chat_id(&self, chat_id: i64, message_type: String) -> Result<Option<Message>>;
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Command,
}

impl AsRef<str> for MessageType {
    fn as_ref(&self) -> &str {
        match self {
            MessageType::Text => "text",
            MessageType::Command => "command",
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Message {
    pub id: i32,
    pub chat_id: i64,
    pub text: String,
    #[sqlx(rename = "type")]
    pub message_type: String,
}

#[derive(sqlx::FromRow)]
pub struct NewMessage {
    pub chat_id: i64,
    pub text: String,
    #[sqlx(rename = "type")]
    pub message_type: String,
}

pub struct MessageRepository<'a> {
    client: &'a PgPool,
}

impl<'a> MessageRepository<'a> {
    pub fn new(client: &'a PgPool) -> Self {
        Self { client }
    }

    pub async fn create_table(&self) -> Result<()> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id              SERIAL PRIMARY KEY,
                chat_id         BIGINT NOT NULL,
                text            TEXT NOT NULL,
                type            TEXT NOT NULL DEFAULT 'text',
                created_at      TIMESTAMP NOT NULL DEFAULT NOW()
            )
        ",
        )
        .execute(self.client)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl<'a> Repository for MessageRepository<'a> {
    async fn insert(&self, value: NewMessage) -> Result<()> {
        sqlx::query("INSERT INTO messages (chat_id, text, type) VALUES ($1, $2, $3)")
            .bind(value.chat_id)
            .bind(value.text)
            .bind(value.message_type)
            .execute(self.client)
            .await?;

        Ok(())
    }

    async fn find_one_by_chat_id(&self, chat_id: i64, message_type: String) -> Result<Option<Message>> {
        let query = "SELECT * FROM messages WHERE chat_id = $1 AND type = $2 ORDER BY created_at DESC LIMIT 1";
        let subscriptions = sqlx::query_as::<_, Message>(query)
            .bind(chat_id)
            .bind(message_type)
            .fetch_all(self.client)
            .await?;

        Ok(subscriptions.into_iter().next())
    }
}
