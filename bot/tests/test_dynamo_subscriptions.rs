#![cfg(feature = "dyndb_int")]
use aws_config::ConfigLoader;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use bot::subscriptions::*;
use itertools::Itertools;
use testcontainers::core::WaitFor;
use testcontainers::*;

#[tokio::test]
async fn testing_subscriptions_dynamodb_access() {
    let docker = clients::Cli::default();
    let image = GenericImage::new("amazon/dynamodb-local", "2.0.0")
        .with_exposed_port(8000)
        .with_wait_for(WaitFor::message_on_stdout("Initializing DynamoDB Local"));
    let node = docker.run(image);
    let dynamodb_port = node.get_host_port_ipv4(8000);

    const CHAT_ID_1: i64 = 123;

    const CHAT_ID_2: i64 = 321;

    let config = ConfigLoader::default()
        .endpoint_url(format!("http://localhost:{dynamodb_port}"))
        .load()
        .await;

    let client = Client::new(&config);

    let request = client
        .create_table()
        .table_name("subscriptions")
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("chat_id") // partition key
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("chat_id")
                .attribute_type(ScalarAttributeType::N)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5) // adjust as necessary
                .write_capacity_units(5) // adjust as necessary
                .build(),
        );
    request
        .send()
        .await
        .expect("failed to create subscriptions");

    let request = client
        .create_table()
        .table_name("subscriptions_inv")
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("addresses") // partition key
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("addresses")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5) // adjust as necessary
                .write_capacity_units(5) // adjust as necessary
                .build(),
        );
    request
        .send()
        .await
        .expect("failed to create subscriptions_inv");

    client
        .append(NewSubscription {
            chat_id: CHAT_ID_1,
            address: "first address".to_string(),
        })
        .await
        .expect("add first address to DB");

    client
        .append(NewSubscription {
            chat_id: CHAT_ID_1,
            address: "second address".to_string(),
        })
        .await
        .expect("add second address to DB");

    client
        .append(NewSubscription {
            chat_id: CHAT_ID_2,
            address: "fist address".to_string(),
        })
        .await
        .expect("add third address to DB");

    let res = client.find_all_by_chat_id(CHAT_ID_1 + 10).await;
    assert!(res.is_err());

    let res = client
        .find_all_by_chat_id(CHAT_ID_1)
        .await
        .expect("failed to get CHAT_ID_1 addresses");
    assert_eq!(res.len(), 2);

    let expected: Vec<String> = vec!["first address".to_string(), "second address".to_string()]
        .into_iter()
        .sorted()
        .collect();
    assert_eq!(
        res.iter()
            .map(|it| it.address.clone())
            .sorted()
            .collect::<Vec<_>>(),
        expected
    );
}
