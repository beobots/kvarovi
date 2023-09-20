use super::models::{NewSubscription, Subscription};
use super::Repository;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem, Update};
use aws_sdk_dynamodb::Client;

static TABLE_NAME: &str = "subscriptions";
static TABLE_NAME_INV: &str = "subscriptions_inv";
static CHAT_ID_FIELD: &str = "chat_id";
static ADDRESSES_FIELD: &str = "addresses";
static CHAT_IDS_FIELD: &str = "chat_ids";

#[async_trait]
impl Repository for Client {
    async fn append(&self, value: NewSubscription) -> Result<()> {
        let update_rev = Update::builder()
            .table_name(TABLE_NAME_INV)
            .key(ADDRESSES_FIELD, AttributeValue::S(value.address.clone()))
            .update_expression(format!("ADD {CHAT_IDS_FIELD} :a"))
            .expression_attribute_values(":a", AttributeValue::Ns(vec![value.chat_id.to_string()]))
            .build();

        let update = Update::builder()
            .table_name(TABLE_NAME)
            .key(CHAT_ID_FIELD, AttributeValue::N(value.chat_id.to_string()))
            .update_expression(format!("ADD {ADDRESSES_FIELD} :a"))
            .expression_attribute_values(":a", AttributeValue::Ss(vec![value.address.clone()]))
            .build();

        let t1 = TransactWriteItem::builder().update(update_rev).build();
        let t2 = TransactWriteItem::builder().update(update).build();

        let request = self
            .transact_write_items()
            .transact_items(t1)
            .transact_items(t2);

        request.send().await?;

        Ok(())
    }

    async fn find_all_by_chat_id(&self, chat_id: i64) -> Result<Vec<Subscription>> {
        let request = self
            .get_item()
            .table_name(TABLE_NAME)
            .key(CHAT_ID_FIELD, AttributeValue::N(chat_id.to_string()));

        let response = request.send().await?;

        if let Some(map) = response.item {
            if let Some(AttributeValue::Ss(addresses)) = map.get(ADDRESSES_FIELD) {
                let res = addresses
                    .iter()
                    .enumerate()
                    .map(|(i, it)| Subscription {
                        id: i as i64,
                        chat_id,
                        address: it.clone(),
                    })
                    .collect::<Vec<_>>();
                return Ok(res);
            } else {
                Err(anyhow!(
                    "\"{ADDRESSES_FIELD}\" attribute is missing in the table \"{TABLE_NAME}\" for ChatId({chat_id})"
                ))
            }
        } else {
            Err(anyhow!(
                "ChatId({chat_id}) is not found in the table \"{TABLE_NAME}\""
            ))
        }
    }

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> anyhow::Result<Vec<Subscription>> {
        todo!()
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> anyhow::Result<()> {
        todo!()
    }
}
