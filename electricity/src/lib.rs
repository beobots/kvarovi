extern crate reqwest;

use reqwest::Error;
use tokio_cron_scheduler::{JobScheduler, Job};
use scraper::{Html, Selector};
use db::{DATABASE, create_table};
use uuid::Uuid;
use aws_sdk_dynamodb::{
    types::{AttributeValue},
    Client, Error as DynamoDBError
};

pub mod db;

struct ShutDownParseData {
    page_url: String,
    city: String,
}

struct TableRowData {
    city: String,
    region: String,
    time: String,
    street: String,
    date: String,
}

async fn fetch_page(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;

    Ok(body)
}

pub async fn collect_data() -> () {
    let dates = vec![
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_0_Iskljucenja.htm"),
            city: String::from("Belgrade")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_1_Iskljucenja.htm"),
            city: String::from("Belgrade")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_2_Iskljucenja.htm"),
            city: String::from("Belgrade")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-beograd/Dan_3_Iskljucenja.htm"),
            city: String::from("Belgrade")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_0_Iskljucenja.htm"),
            city: String::from("Novi Sad")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_1_Iskljucenja.htm"),
            city: String::from("Novi Sad")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_2_Iskljucenja.htm"),
            city: String::from("Novi Sad")
        },
        ShutDownParseData {
            page_url: String::from("https://elektrodistribucija.rs/planirana-iskljucenja-srbija/NoviSad_Dan_3_Iskljucenja.htm"),
            city: String::from("Novi Sad")
        }
    ];

    let mut table_rows: Vec<TableRowData> = vec![];

    for input in dates {
        let html = fetch_page(&input.page_url).await.unwrap();
        let document = Html::parse_document(&html);
        let table_selector = Selector::parse("table").unwrap();
        let mut tables = document.select(&table_selector);
        
        let header_table = tables.nth(0).unwrap();
        let header_selector = Selector::parse("tbody > tr > td > b").unwrap();
        let header = header_table.select(&header_selector).next().unwrap();
        let date = header.text().collect::<Vec<_>>().last().unwrap().to_string().split(" ").last().unwrap().to_string();

        let table = tables.nth(0).unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        for tr_element in table.select(&tr_selector).skip(1) {
            let mut td_elements = tr_element.select(&td_selector);
            let region: String = td_elements.nth(0).unwrap().text().collect::<Vec<_>>().into_iter().collect();
            let time: String = td_elements.nth(0).unwrap().text().collect::<Vec<_>>().into_iter().collect();
            let street: String = td_elements.nth(0).unwrap().text().collect::<Vec<_>>().into_iter().collect();

            println!("{}", region);
            println!("{}", time);
            println!("{}", street);

            table_rows.push(
                TableRowData { region, time, street, city: input.city.clone(), date: date.clone() }
            )           
        }
    }

    let db_client = unsafe {
        DATABASE.get_client().await
    };

    let _ = create_table(db_client, "electricity_failures", "id").await;

    for table_row in table_rows {
        let id = Uuid::new_v4();
        let _ = add_electricity_failure_item(db_client, id.to_string(), table_row).await;
    }
}

pub async fn get_collect_data_job() -> Job {
    Job::new("1/10 * * * * *", |uuid, l| {
        println!("adasdgasdg")
    }).unwrap()
}

pub async fn start_scheduler() -> () {
    let scheduler = JobScheduler::new().await.unwrap();
    let collect_data_job = get_collect_data_job().await;

    let _ = scheduler.add(collect_data_job).await;
    let _ = scheduler.start().await;
}

async fn add_electricity_failure_item(client: &Client, id: String, item: TableRowData) -> Result<(), DynamoDBError> {
    let city_av = AttributeValue::S(item.city);
    let region_av = AttributeValue::S(item.region);
    let time_av = AttributeValue::S(item.time);
    let street_av = AttributeValue::S(item.street);
    let date_av = AttributeValue::S(item.date);

    let request = client
        .put_item()
        .table_name("electricity_failures")
        .item("id", AttributeValue::S(id))
        .item("city", city_av)
        .item("region", region_av)
        .item("time", time_av)
        .item("street", street_av)
        .item("date", date_av);

    let _ = request.send().await?;

    Ok(())
}
