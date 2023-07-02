use anyhow::{Ok, Result};
#[cfg(not(feature = "lambda"))]
use dotenvy::dotenv;

#[cfg(feature = "lambda")]
use lambda_runtime::{service_fn, LambdaEvent};

#[cfg(feature = "lambda")]
use serde_json::Value;

use electricity::db::init_client;

#[cfg(not(feature = "lambda"))]
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect(".env file not found");

    let db_client = init_client().await?;
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
    electricity::collect_data(&db_client, &pages).await?;

    Ok(())
}

#[cfg(feature = "lambda")]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(my_handler);
    if let Err(e) = lambda_runtime::run(func).await {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(feature = "lambda")]
pub(crate) async fn my_handler(_: LambdaEvent<Value>) -> Result<()> {
    let db_client = init_client().await?;
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
    electricity::collect_data(&db_client, &pages).await?;

    Ok(())
}
