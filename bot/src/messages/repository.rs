use super::models::{Message, MessageType};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    async fn append(&self, message: Message) -> Result<()>;
    async fn find_last(&self, chat_id: i64, message_type: MessageType) -> Result<Option<Message>>;
}
