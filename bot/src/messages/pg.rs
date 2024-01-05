use super::models::{Message, MessageType};
use super::repository::Repository;
use sqlx::PgPool;

impl Repository for PgPool {
    async fn append(&self, message: Message) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO messages (chat_id, text, type) VALUES ($1, $2, $3)")
            .bind(message.chat_id)
            .bind(message.text)
            .bind(message.message_type)
            .execute(self)
            .await?;

        Ok(())
    }

    async fn find_last(&self, chat_id: i64, message_type: MessageType) -> anyhow::Result<Option<Message>> {
        let subscriptions = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE chat_id = $1 AND type = $2 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(chat_id)
        .bind(message_type)
        .fetch_all(self)
        .await?;

        Ok(subscriptions.into_iter().next())
    }
}
