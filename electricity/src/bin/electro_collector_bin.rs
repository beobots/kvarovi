//! An utility to run electro collection right from the command line.
use anyhow::{Ok, Result};
use dotenvy::dotenv;
use electricity::db::init_custom_client;
use electricity::BEOGRAD_ELECTRICITY_PAGES;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect(".env file not found");

    let db_client = init_custom_client().await?;
    let raw_data_table_name = env::var("RAW_DATA_TABLE_NAME").unwrap_or("electricity_failures_raw".to_owned());

    electricity::collect_data(&db_client, &raw_data_table_name, BEOGRAD_ELECTRICITY_PAGES).await?;

    Ok(())
}
