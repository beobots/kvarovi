//! Lambda function that downloads electricity time table and
//! stores it into the database.
use anyhow::Result;
use electricity::db::init_client;
use electricity::BEOGRAD_ELECTRICITY_PAGES;
use lambda_runtime::{service_fn, LambdaEvent};
use serde_json::Value;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let trace_level = env::var("TRACE_LVL").unwrap_or("INFO".to_owned());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&trace_level).unwrap_or(tracing::Level::INFO))
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(electro_handler);
    if let Err(e) = lambda_runtime::run(func).await {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn electro_handler(_: LambdaEvent<Value>) -> Result<()> {
    let db_client = init_client().await?;
    let raw_data_table_name = env::var("RAW_DATA_TABLE_NAME")?;
    electricity::collect_data(&db_client, &raw_data_table_name, BEOGRAD_ELECTRICITY_PAGES).await
}
