use anyhow::{Ok, Result};
use aws_sdk_dynamodb::{
    config::{Config, Region},
    types::{AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType},
    Client,
};
use std::env;

// TODO - this does not look right
async fn make_custom_config() -> Result<Config> {
    let database_url = env::var("DATABASE_URL").unwrap_or("http://localhost:8000".to_owned());
    let profile_name = env::var("AWS_PROFILE")?;
    let region = env::var("AWS_REGION").unwrap_or("eu-central-1".to_owned());

    let config = aws_config::from_env()
        .profile_name(&profile_name)
        .region(Region::new(region))
        .load()
        .await;

    Ok(aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(&database_url)
        .build())
}

pub async fn init_custom_client() -> Result<Client> {
    let config = make_custom_config().await?;
    Ok(Client::from_conf(config))
}

async fn make_config() -> Result<Config> {
    let config = aws_config::from_env().load().await;

    Ok(aws_sdk_dynamodb::config::Builder::from(&config).build())
}

pub async fn init_client() -> Result<Client> {
    let config = make_config().await?;
    Ok(Client::from_conf(config))
}

pub async fn check_table_exists(client: &Client, name: &str) -> Result<bool> {
    let tables = client.list_tables().send().await?;
    let exists = tables.table_names().into_iter().any(|table| {
        if let Some(table_name) = table.get(0) {
            return table_name.to_owned().eq(name);
        }

        false
    });

    Ok(exists)
}

pub async fn create_table(client: &Client, table: &str, key: &str) -> Result<()> {
    let table_exists = check_table_exists(client, table).await?;

    if table_exists {
        return Ok(());
    }

    let a_name: String = key.into();
    let table_name: String = table.into();

    let ad = AttributeDefinition::builder()
        .attribute_name(&a_name)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(&a_name)
        .key_type(KeyType::Hash)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    let _ = client
        .create_table()
        .table_name(table_name)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await?;

    Ok(())
}
