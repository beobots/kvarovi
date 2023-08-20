use std::fmt::Display;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::postgres::PgPool;

#[async_trait]
pub trait Repository<'a, T> {
    type WhereInput;
    type NewValue;

    fn new(client: &'a PgPool) -> Self;
    async fn insert(&self, value: Self::NewValue) -> Result<()>;
    async fn find_all(&self, where_input: Self::WhereInput, args: Option<FindAllArgs>) -> Result<Vec<T>>;
    async fn find_one(&self, where_input: Self::WhereInput, args: Option<FindOneArgs>) -> Result<Option<T>>;
    async fn delete(&self, where_input: Self::WhereInput) -> Result<()>;

    fn format_where_clause(where_input: Self::WhereInput) -> String;
}

#[derive(Clone, Copy, Default)]
pub struct WhereValue<'a, T> {
    pub eq: Option<T>,
    pub in_array: Option<&'a Vec<T>>,
}

#[derive(Clone, Copy)]
pub enum Order {
    ASC,
    DESC,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order = match self {
            Order::ASC => "ASC",
            Order::DESC => "DESC",
        };

        write!(f, "{}", order)
    }
}

#[derive(Clone, Default)]
pub struct FindAllArgs {
    pub limit: Option<i64>,
    pub order_by: Option<(String, Order)>,
    // pub offset: Option<i64>,
}

#[derive(Default)]
pub struct FindOneArgs {
    pub order_by: Option<(String, Order)>,
}

impl From<FindOneArgs> for FindAllArgs {
    fn from(args: FindOneArgs) -> Self {
        Self {
            limit: Some(1),
            order_by: args.order_by,
        }
    }
}
