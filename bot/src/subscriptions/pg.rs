use super::models::{NewSubscription, Subscription};
use super::Repository;
use anyhow::Result;
use itertools::Itertools;
use sqlx::PgPool;

impl Repository for PgPool {
    async fn append(&self, value: NewSubscription) -> Result<()> {
        sqlx::query("INSERT INTO subscriptions (chat_id, address) values($1, $2)")
            .bind(value.chat_id)
            .bind(value.address)
            .execute(self)
            .await?;

        Ok(())
    }

    async fn find_all_by_chat_id(&self, chat_id: i64) -> Result<Vec<Subscription>> {
        let subs: Vec<_> = sqlx::query_as("SELECT id, chat_id, address FROM subscriptions WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_all(self)
            .await?;

        Ok(subs)
    }

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> Result<Vec<Subscription>> {
        let addresses = addresses
            .iter()
            .map(|address| format!("'%{}%'", address))
            .join(", ");

        let query = format!("SELECT * FROM subscriptions WHERE address ilike any (array[{addresses}])");
        let subscriptions = sqlx::query_as::<_, Subscription>(query.as_str())
            .fetch_all(self)
            .await?;

        Ok(subscriptions)
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> Result<()> {
        let ids_string = ids.into_iter().map(|it| it.to_string()).join(",");
        let query = format!("DELETE FROM subscriptions WHERE ID in ({ids_string})");

        sqlx::query(&query).execute(self).await?;

        Ok(())
    }
}
