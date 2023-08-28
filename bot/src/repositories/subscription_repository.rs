use anyhow::{Ok, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPool;

#[async_trait]
pub trait Repository {
    async fn insert(&self, value: NewSubscription) -> Result<()>;
    async fn find_all_by_chat_id(&self, chat_id: i64) -> Result<Vec<Subscription>>;
    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> Result<Vec<Subscription>>;
    async fn delete_by_ids(&self, ids: Vec<i64>) -> Result<()>;
}

#[derive(sqlx::FromRow)]
pub struct Subscription {
    pub id: i32,
    pub chat_id: i64,
    pub address: String,
}

pub struct NewSubscription {
    pub chat_id: i64,
    pub address: String,
}

pub struct SubscriptionsRepository<'a> {
    client: &'a PgPool,
}

impl<'a> SubscriptionsRepository<'a> {
    pub fn new(client: &'a PgPool) -> Self {
        Self { client }
    }

    pub async fn create_table(&self) -> Result<()> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS subscriptions (
                id              SERIAL PRIMARY KEY,
                chat_id         BIGINT NOT NULL,
                address         TEXT NOT NULL,
                created_at      TIMESTAMP NOT NULL DEFAULT NOW()
            )
        ",
        )
        .execute(self.client)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl<'a> Repository for SubscriptionsRepository<'a> {
    async fn insert(&self, value: NewSubscription) -> Result<()> {
        let query = "INSERT INTO subscriptions (chat_id, address) VALUES ($1, $2)";

        sqlx::query(query)
            .bind(value.chat_id)
            .bind(value.address)
            .execute(self.client)
            .await?;

        Ok(())
    }

    async fn find_all_by_chat_id(&self, chat_id: i64) -> Result<Vec<Subscription>> {
        let query = "SELECT * FROM subscriptions WHERE chat_id = $1";
        let subscriptions = sqlx::query_as::<_, Subscription>(query)
            .bind(chat_id)
            .fetch_all(self.client)
            .await?;

        Ok(subscriptions)
    }

    async fn delete_by_ids(&self, ids: Vec<i64>) -> Result<()> {
        let ids = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let query = format!("DELETE FROM subscriptions WHERE id IN ({})", ids);

        sqlx::query(query.as_str()).execute(self.client).await?;

        Ok(())
    }

    async fn find_all_by_addresses(&self, addresses: Vec<String>) -> Result<Vec<Subscription>> {
        let addresses = addresses
            .iter()
            .map(|address| format!("'%{}%'", address))
            .collect::<Vec<String>>()
            .join(", ");
        let query = format!(
            "SELECT * FROM subscriptions WHERE address ilike any (array[{}])",
            addresses
        );
        let subscriptions = sqlx::query_as::<_, Subscription>(query.as_str())
            .fetch_all(self.client)
            .await?;

        Ok(subscriptions)
    }
}
