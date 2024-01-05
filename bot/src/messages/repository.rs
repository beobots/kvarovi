use super::models::{Message, MessageType};
use anyhow::Result;
use std::future::Future;

pub trait Repository {
    fn append(&self, message: Message) -> impl Future<Output = Result<()>> + Send;
    fn find_last(
        &self,
        chat_id: i64,
        message_type: MessageType,
    ) -> impl Future<Output = Result<Option<Message>>> + Send;
}
