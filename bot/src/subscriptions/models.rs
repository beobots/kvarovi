#[derive(sqlx::FromRow, Debug)]
pub(crate) struct NewSubscription {
    pub chat_id: i64,
    pub address: String,
}

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct SubId(i64);

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct Subscription {
    pub id: i64,
    pub chat_id: i64,
    pub address: String,
}
