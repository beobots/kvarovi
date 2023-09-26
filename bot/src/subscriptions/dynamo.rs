use super::models::{NewSubscription, Subscription};
use super::Repository;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ComparisonOperator, Condition, TransactWriteItem, Update};
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

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> Result<Vec<Subscription>> {
        let addresses_attrs = addresses
            .into_iter()
            .map(|it| AttributeValue::S(it))
            .collect();

        let cond = Condition::builder()
            .comparison_operator(ComparisonOperator::Eq)
            .set_attribute_value_list(Some(addresses_attrs))
            .build();

        let request = self
            .query()
            .table_name(TABLE_NAME_INV)
            .key_conditions(ADDRESSES_FIELD, cond);

        let response = request.send().await?;

        let items = response.items.unwrap_or_default();
        let mut result = Vec::new();
        for item in items {
            let address = if let Some(AttributeValue::S(address)) = item.get(ADDRESSES_FIELD) {
                address.clone()
            } else {
                bail!("\"{ADDRESSES_FIELD}\" field is missing in the table \"{TABLE_NAME_INV}\"");
            };

            if let Some(AttributeValue::Ns(chat_ids)) = item.get(CHAT_IDS_FIELD) {
                result.extend(
                    chat_ids
                        .iter()
                        .flat_map(|it| it.parse::<i64>().ok())
                        .enumerate()
                        .map(|(i, it)| Subscription {
                            id: it + i as i64,
                            chat_id: it,
                            address: address.clone(),
                        }),
                );
            } else {
                bail!("\"{CHAT_IDS_FIELD}\" field is missing in the table \"{TABLE_NAME_INV}\"");
            }
        }

        Ok(result)
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> anyhow::Result<()> {
        todo!()
    }
}
