use super::models::{NewSubscription, Subscription};
use anyhow::Result;
use std::future::Future;

pub trait Repository {
    /// Appends new subscription to the user's list.
    fn append(&self, value: NewSubscription) -> impl Future<Output = Result<()>> + Send;
    fn find_all_by_chat_id(&self, chat_id: i64) -> impl Future<Output = Result<Vec<Subscription>>> + Send;
    fn find_all_by_addresses(&self, addresses: Vec<String>) -> impl Future<Output = Result<Vec<Subscription>>> + Send;
    fn delete_by_ids(&self, ids: Vec<i64>) -> impl Future<Output = Result<()>> + Send;
}
