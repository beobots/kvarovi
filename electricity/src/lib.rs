extern crate reqwest;

use reqwest::Error;

mod addresses;
mod time_interval;
mod translit;

struct ShutDownParseData {
    days_number: i8,
    page_url: String,
    city: String,
}

pub async fn console_lib() -> Result<(), Error> {
    let dates = vec![
        ShutDownParseData {
            days_number: 0,
            page_url: String::from(
                "https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_0_Iskljucenja.htm",
            ),
            city: String::from("Belgrade"),
        },
        ShutDownParseData {
            days_number: 0,
            page_url: String::from(
                "https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_1_Iskljucenja.htm",
            ),
            city: String::from("Belgrade"),
        },
        ShutDownParseData {
            days_number: 0,
            page_url: String::from(
                "https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_2_Iskljucenja.htm",
            ),
            city: String::from("Belgrade"),
        },
        ShutDownParseData {
            days_number: 0,
            page_url: String::from(
                "https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_3_Iskljucenja.htm",
            ),
            city: String::from("Belgrade"),
        },
    ];

    for date in dates {
        let html = fetch_page(&date.page_url).await?;

        println!("{}", html);
    }

    Ok(())
}

async fn fetch_page(url: &str) -> Result<String, Error> {
    let response = reqwest::get(&*url).await?;
    println!("Status: {}", response.status());

    let body = response.text().await?;
    println!("Body:\n{}", body);

    Ok(body)
}
