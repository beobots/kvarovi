use anyhow::{Ok, Result};

use dotenvy::dotenv;
use electricity::db::init_client;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect(".env file not found");

    let db_client = init_client().await?;

    let raw_data_table_name = env::var("RAW_DATA_TABLE_NAME").unwrap_or("electricity_failures_raw".to_owned());
    // let data_table_name =
    // env::var("DATA_TABLE_NAME").unwrap_or("electricity_failures".to_owned());

    let pages = vec![
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_0_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_1_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_2_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_3_Iskljucenja.htm"),
        //     String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_0_Iskljucenja.htm"),
        //     String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_1_Iskljucenja.htm"),
        //     String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_2_Iskljucenja.htm"),
        //     String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_3_Iskljucenja.htm"),
    ];
    electricity::collect_data(&db_client, &raw_data_table_name, &pages).await?;

    Ok(())
}
