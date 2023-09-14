use super::models::{ChatPreference, Language};
use super::repository::Repository;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::postgres::PgPool;

pub struct PgChatPreference<'a> {
    client: &'a PgPool,
}

impl<'a> PgChatPreference<'a> {
    pub fn new(client: &'a PgPool) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<'a> Repository for PgChatPreference<'a> {
    async fn insert(&self, value: ChatPreference) -> Result<()> {
        sqlx::query("INSERT INTO preference (chat_id, language) VALUES ($1, $2)")
            .bind(value.chat_id)
            .bind(value.language)
            .execute(self.client)
            .await?;

        Ok(())
    }

    async fn find_one(&self, chat_id: i64) -> Result<Option<ChatPreference>> {
        let query = String::from("SELECT * FROM preference WHERE chat_id = $1");
        let subscriptions = sqlx::query_as::<_, ChatPreference>(query.as_str())
            .bind(chat_id)
            .fetch_all(self.client)
            .await?;

        Ok(subscriptions.into_iter().next())
    }

    async fn update_language(&self, chat_id: i64, language: Language) -> Result<()> {
        let query = "UPDATE preference SET language = $1, updated_at = NOW() WHERE chat_id = $2";

        sqlx::query(query)
            .bind(language)
            .bind(chat_id)
            .execute(self.client)
            .await?;

        Ok(())
    }
}
