use super::models::{NewSubscription, Subscription};
use super::Repository;
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_dynamodb::operation::update_item::UpdateItemInput;
use aws_sdk_dynamodb::types::builders::UpdateBuilder;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use itertools::Update;
use teloxide_core::types::MenuButton::Default;
use uuid::Uuid;

static TABLE_NAME: &str = "subscriptions";
static TABLE_NAME_REV: &str = "subscriptions_rev";
static CHAT_ID_FIELD: &str = "chat_id";
static ADDRESS_FIELD: &str = "address";

#[async_trait]
impl Repository for Client {
    async fn append(&self, value: NewSubscription) -> Result<()> {
        let update_rev = Update::builder()
            .table_name(TABLE_NAME_REV)
            .key(ADDRESS_FIELD, AttributeValue::S(value.address))
            .update_expression("ADD chat_ids :a")
            .expression_attribute_values(":a", AttributeValue::S(value.chat_id.to_string()))
            .build()?;

        let update = Update::builder()
            .table_name(TABLE_NAME)
            .key(CHAT_ID_FIELD, AttributeValue::S(value.chat_id.to_string()))
            .update_expression("ADD addresses :a")
            .expression_attribute_values(":a", AttributeValue::S(value.address.to_string()));

        let t1 = TransactWriteItem::builder().update(update_rev).build();
        let t2 = TransactWriteItem::builder().update(update).build();

        let request = self
            .transact_write_items()
            .transact_items(t1)
            .transact_items(t2);

        request.send().await?;

        Ok(())
    }

    async fn find_all_by_chat_id(&self, chat_id: i64) -> anyhow::Result<Vec<Subscription>> {
        todo!()
    }

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> anyhow::Result<Vec<Subscription>> {
        todo!()
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> anyhow::Result<()> {
        todo!()
    }
}
