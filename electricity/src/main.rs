use dotenvy::dotenv;
use electricity::db::{init_client};

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let db_client = init_client().await;
    let pages = vec![
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_0_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_1_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_2_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_3_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_0_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_1_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_2_Iskljucenja.htm"),
        String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_3_Iskljucenja.htm")
    ];
    let _ = electricity::collect_data(&db_client, &pages).await;
}
