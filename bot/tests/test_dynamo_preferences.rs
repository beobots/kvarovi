#![cfg(feature = "dyndb_int")]
use aws_config::ConfigLoader;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use bot::preferences::Language;
use bot::preferences::*;
use testcontainers::core::WaitFor;
use testcontainers::*;

#[tokio::test]
async fn testing_dynamodb() {
    let docker = clients::Cli::default();
    let image = GenericImage::new("amazon/dynamodb-local", "2.0.0")
        .with_exposed_port(8000)
        .with_wait_for(WaitFor::message_on_stdout("Initializing DynamoDB Local"));
    let node = docker.run(image);
    let dynamodb_port = node.get_host_port_ipv4(8000);

    const CHAT_ID: i64 = 123;

    let config = ConfigLoader::default()
        .endpoint_url(format!("http://localhost:{dynamodb_port}"))
        .load()
        .await;

    let client = Client::new(&config);

    let request = client
        .create_table()
        .table_name("chat_preferences")
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
        .expect("failed to create a dynamodb table");

    client
        .insert(ChatPreference {
            chat_id: CHAT_ID,
            language: Language::Ru,
        })
        .await
        .expect("failed to insert a value into dynamodb table");

    let Some(res) = client
        .find_one_by_chat_id(CHAT_ID)
        .await
        .expect("failed to connect to dynamodb to find preferences")
    else {
        panic!("failed to find chat preferences")
    };
    assert_eq!(res.language, Language::Ru);

    client
        .update_language(CHAT_ID, Language::Rs)
        .await
        .expect("failed to update preferences");

    let Some(res) = client
        .find_one_by_chat_id(CHAT_ID)
        .await
        .expect("failed to connect to dynamodb to find updated preferences")
    else {
        panic!("failed to find chat preferences for the updated record")
    };
    assert_eq!(res.language, Language::Rs);
}
