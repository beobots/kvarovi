#[derive(sqlx::FromRow, Debug)]
pub struct NewSubscription {
    pub chat_id: i64,
    pub address: String,
}

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct SubId(i64);

#[derive(sqlx::FromRow, Debug)]
pub struct Subscription {
    pub id: i64,
    pub chat_id: i64,
    pub address: String,
}
