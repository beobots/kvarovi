use super::models::{NewSubscription, Subscription};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    /// Appends new subscription to the user's list.
    async fn append(&self, value: NewSubscription) -> Result<()>;
    async fn find_all_by_chat_id(&self, chat_id: i64) -> Result<Vec<Subscription>>;
    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> Result<Vec<Subscription>>;
    async fn delete_by_ids(&self, ids: Vec<i64>) -> Result<()>;
}
