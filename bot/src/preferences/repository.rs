use super::models::ChatPreference;
use crate::preferences::Language;
use anyhow::Result;
use std::future::Future;

pub trait Repository {
    fn insert(&self, value: ChatPreference) -> impl Future<Output = Result<()>>;
    fn find_one(&self, chat_id: i64) -> impl Future<Output = Result<Option<ChatPreference>>>;
    fn update_language(&self, chat_id: i64, language: Language) -> impl Future<Output = Result<()>>;
}
