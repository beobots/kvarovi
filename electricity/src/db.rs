use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, ProvisionedThroughput, KeySchemaElement,
        KeyType, ScalarAttributeType
    },
    config::{Region, Config},
    operation::{create_table::CreateTableOutput},
    Client, Error,
};

pub async fn make_config() -> Config {
    let config = aws_config::from_env().profile_name("localstack").region(Region::new("us-west-2")).load().await;

    aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(
            "http://localhost:8000",
        )
        .build()
}

pub struct Database {
    client: Option<Client>
}

impl Database {
    pub async fn get_client(&mut self) -> &Client {
        if self.client.is_none() {
            let config = make_config().await;
            let client = Client::from_conf(config);
            self.client = Some(client);
        }

        self.client.as_ref().unwrap()
    }
}

pub static mut DATABASE: Database = Database {
    client: None,
};

pub async fn create_table(
    client: &Client,
    table: &str,
    key: &str,
) -> Result<CreateTableOutput, Error> {
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

    let create_table_response = client
        .create_table()
        .table_name(table_name)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await;

    match create_table_response {
        Ok(out) => {
            println!("Added table {} with key {}", table, key);
            Ok(out)
        }
        Err(e) => {
            eprintln!("Got an error creating table:");
            eprintln!("{}", e);
            Err(e.into())
        }
    }
}