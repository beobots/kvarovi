use super::models::{Message, MessageType};
use super::repository::Repository;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;

const TABLE_NAME: &str = "messages";
const ID_FIELD: &str = "id";
const TEXT_FIELD: &str = "text";
const MESSAGE_TYPE: &str = "message_type";

#[async_trait]
impl Repository for Client {
    async fn append(&self, message: Message) -> Result<()> {
        let request = self
            .put_item()
            .table_name(TABLE_NAME)
            .item(ID_FIELD, make_id(message.chat_id, message.message_type))
            .item(TEXT_FIELD, AttributeValue::S(message.text.to_owned()))
            .item(
                MESSAGE_TYPE,
                AttributeValue::S(message.message_type.as_ref().to_string()),
            );

        let _ = request.send().await?;

        Ok(())
    }

    async fn find_last(&self, chat_id: i64, message_type: MessageType) -> Result<Option<Message>> {
        let request = self
            .get_item()
            .table_name(TABLE_NAME)
            .key(ID_FIELD, make_id(chat_id, message_type));

        let resp = request.send().await?;

        if let Some(item) = resp.item {
            if let Some(AttributeValue::S(attr_value)) = item.get(TEXT_FIELD) {
                Ok(Some(Message {
                    chat_id,
                    text: attr_value.clone(),
                    message_type,
                }))
            } else {
                Err(anyhow!("language field is missing for {chat_id}"))
            }
        } else {
            Ok(None)
        }
    }
}

fn make_id(chat_id: i64, message_type: MessageType) -> AttributeValue {
    AttributeValue::S(format!("{}-{}", chat_id, message_type))
}
