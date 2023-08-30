use super::models::{ChatPreference, NewChatPreference};
use crate::preferences::Language;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    async fn insert(&self, value: NewChatPreference) -> Result<()>;
    async fn find_one_by_chat_id(&self, chat_id: i64) -> Result<Option<ChatPreference>>;
    async fn update_language(&self, chat_id: i64, language: Language) -> Result<()>;
}
