use super::Repository;
use crate::subscriptions::models::{NewSubscription, Subscription};
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_dynamodb::operation::delete_item::DeleteItemInput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use uuid::uuid;

static TABLE_NAME: &str = "subscriptions";

static KEY: &str = "id";

#[async_trait]
impl Repository for Client {
    async fn append(&self, value: NewSubscription) -> Result<()> {
        // self.put_item().table_name(TABLE_NAME).item();
        todo!()
    }

    async fn find_all_by_chat_id(&self, chat_id: i64) -> anyhow::Result<Vec<Subscription>> {
        let request = self
            .get_item()
            .table_name(TABLE_NAME)
            .key(KEY, AttributeValue::S(chat_id.to_string()));

        todo!()
    }

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> anyhow::Result<Vec<Subscription>> {
        todo!()
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> anyhow::Result<()> {
        for id in ids {
            let request = self
                .delete_item()
                .table_name(TABLE_NAME)
                .key(KEY, AttributeValue::S(id.to_string()));

            request.send().await?;
        }

        Ok(())
    }
}
