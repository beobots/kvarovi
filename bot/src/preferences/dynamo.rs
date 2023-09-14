use super::repository::Repository;
use crate::preferences::{ChatPreference, Language};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use std::str::FromStr;

const TABLE_NAME: &str = "chat_preferences";

const CHAT_ID_FIELD: &str = "chat_id";

const LANGUAGE_FIELD: &str = "lang";

const VALUE_EXPR: &str = ":value";

#[async_trait]
impl Repository for Client {
    async fn insert(&self, value: ChatPreference) -> Result<()> {
        let request = self
            .put_item()
            .table_name(TABLE_NAME)
            .item(CHAT_ID_FIELD, AttributeValue::N(value.chat_id.to_string()))
            .item(
                LANGUAGE_FIELD,
                AttributeValue::S(value.language.as_ref().to_string()),
            );

        let _ = request.send().await?;

        Ok(())
    }

    async fn find_one(&self, chat_id: i64) -> Result<Option<ChatPreference>> {
        let request = self
            .get_item()
            .table_name(TABLE_NAME)
            .key(CHAT_ID_FIELD, AttributeValue::N(chat_id.to_string()));

        let resp = request.send().await?;

        if let Some(item) = resp.item {
            if let Some(AttributeValue::S(attr_value)) = item.get(LANGUAGE_FIELD) {
                Ok(Some(ChatPreference {
                    chat_id,
                    language: Language::from_str(attr_value)?,
                }))
            } else {
                Err(anyhow!("language field is missing for {chat_id}"))
            }
        } else {
            Ok(None)
        }
    }

    async fn update_language(&self, chat_id: i64, language: Language) -> anyhow::Result<()> {
        let request = self
            .update_item()
            .table_name(TABLE_NAME)
            .key(CHAT_ID_FIELD, AttributeValue::N(chat_id.to_string()))
            .update_expression(format!("SET {LANGUAGE_FIELD}={VALUE_EXPR}"))
            .expression_attribute_values(VALUE_EXPR, AttributeValue::S(language.as_ref().to_string()));

        let _ = request.send().await?;

        Ok(())
    }
}
