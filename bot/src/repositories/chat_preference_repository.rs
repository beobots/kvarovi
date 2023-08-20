use anyhow::{Ok, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPool;

#[async_trait]
pub trait Repository {
    async fn insert(&self, value: NewChatPreference) -> Result<()>;
    async fn find_one_by_chat_id(&self, chat_id: i64) -> Result<Option<ChatPreference>>;
    async fn update_language(&self, chat_id: i64, language: String) -> Result<()>;
}

#[derive(sqlx::FromRow)]
pub struct ChatPreference {
    pub id: i32,
    pub chat_id: i64,
    pub language: String,
}

pub struct NewChatPreference {
    pub chat_id: i64,
    pub language: String,
}

pub struct ChatPreferenceRepository<'a> {
    client: &'a PgPool,
}

impl<'a> ChatPreferenceRepository<'a> {
    pub fn new(client: &'a PgPool) -> Self {
        Self { client }
    }

    pub async fn create_table(&self) -> Result<()> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS preference (
                id              SERIAL PRIMARY KEY,
                chat_id         BIGINT NOT NULL,
                language        TEXT NOT NULL,
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
impl<'a> Repository for ChatPreferenceRepository<'a> {
    async fn insert(&self, value: NewChatPreference) -> Result<()> {
        sqlx::query("INSERT INTO preference (chat_id, language) VALUES ($1, $2)")
            .bind(value.chat_id)
            .bind(value.language)
            .execute(self.client)
            .await?;

        Ok(())
    }

    async fn find_one_by_chat_id(&self, chat_id: i64) -> Result<Option<ChatPreference>> {
        let query = String::from("SELECT * FROM preference WHERE chat_id = $1");
        let subscriptions = sqlx::query_as::<_, ChatPreference>(query.as_str())
            .bind(chat_id)
            .fetch_all(self.client)
            .await?;

        Ok(subscriptions.into_iter().next())
    }

    async fn update_language(&self, chat_id: i64, language: String) -> Result<()> {
        let query = "UPDATE preference SET language = $1 WHERE chat_id = $2";

        sqlx::query(query)
            .bind(language)
            .bind(chat_id)
            .execute(self.client)
            .await?;

        Ok(())
    }
}
