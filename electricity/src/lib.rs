extern crate reqwest;

use anyhow::{Ok, Result};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::NaiveDate;
use db::create_table;
use scraper::{Html, Selector};
use uuid::Uuid;

pub mod db;

struct TableRowData {
    city: String,
    region: String,
    time: String,
    street: String,
    date: String,
}

async fn fetch_page(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}

pub async fn collect_data(db_client: &Client, pages: &[String]) {
    let mut table_rows: Vec<TableRowData> = vec![];

    let requests = pages.iter().map(|page| {
        let p = page.clone();
        tokio::spawn(async move { request_page(p).await })
    });

    let results = futures::future::join_all(requests).await;

    for result in results {
        table_rows.append(&mut result.unwrap())
    }

    let _ = create_table(db_client, "electricity_failures", "id").await;

    for table_row in table_rows {
        let _ = add_electricity_failure_item(db_client, &table_row).await;
    }
}

async fn request_page(page: String) -> Vec<TableRowData> {
    let html = fetch_page(&page).await.unwrap();
    let rows: Vec<TableRowData> = parse_page_to_rows(html);

    rows
}

fn format_date(date: String) -> String {
    let mut naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d");

    if naive_date.is_err() {
        naive_date = NaiveDate::parse_from_str(&date, "%d-%m-%Y");
    }

    naive_date.unwrap().format("%d-%m-%Y").to_string()
}

fn parse_page_to_rows(page_html: String) -> Vec<TableRowData> {
    let document = Html::parse_document(&page_html);
    let table_selector = Selector::parse("table").unwrap();
    let tables = document.select(&table_selector).collect::<Vec<_>>();

    let header_table = tables.get(0).unwrap();
    let header_selector = Selector::parse("tbody > tr > td > b").unwrap();
    let header: String = header_table
        .select(&header_selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .into_iter()
        .collect();
    let date = header
        .split(' ')
        .last()
        .unwrap()
        .to_owned()
        .split('.')
        .filter(|&str| !str.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let city = header
        .split(" - ")
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .to_string();

    let table = tables.get(1).unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut table_rows: Vec<TableRowData> = vec![];

    let rows = table.select(&tr_selector).collect::<Vec<_>>();
    let heading_row = rows.get(0).unwrap();
    let heading_td = heading_row.select(&td_selector);

    let street_index = heading_td
        .clone()
        .position(|cell| {
            let title: String = cell.text().collect::<Vec<_>>().into_iter().collect();

            title == "Улице"
        })
        .unwrap();
    let time_index = heading_td
        .clone()
        .position(|cell| {
            let title: String = cell.text().collect::<Vec<_>>().into_iter().collect();

            title == "Време"
        })
        .unwrap();
    let region_index = heading_td
        .clone()
        .position(|cell| {
            let title: String = cell.text().collect::<Vec<_>>().into_iter().collect();

            title == "Општина"
        })
        .unwrap();

    for tr_element in &rows[1..] {
        let td_elements = tr_element.select(&td_selector).collect::<Vec<_>>();
        let region: String = td_elements
            .get(region_index)
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let time: String = td_elements
            .get(time_index)
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let street: String = td_elements
            .get(street_index)
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect();

        table_rows.push(TableRowData {
            region,
            time,
            street,
            city: city.clone(),
            date: format_date(date.clone()),
        })
    }

    table_rows
}

async fn add_electricity_failure_item(client: &Client, item: &TableRowData) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let city_av = AttributeValue::S(item.city.to_owned());
    let region_av = AttributeValue::S(item.region.to_owned());
    let time_av = AttributeValue::S(item.time.to_owned());
    let street_av = AttributeValue::S(item.street.to_owned());
    let date_av = AttributeValue::S(item.date.to_owned());

    // let res = find_electricity_failure_item(client, &item).await?;

    // if let Some(q) = res {
    //     println!("{}", q);
    // }

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

#[allow(dead_code)]
async fn find_electricity_failure_item(client: &Client, item: &TableRowData) -> Result<Option<String>> {
    let city_av = AttributeValue::S(item.city.to_owned());
    let region_av = AttributeValue::S(item.region.to_owned());
    let time_av = AttributeValue::S(item.time.to_owned());
    let date_av = AttributeValue::S(item.date.to_owned());

    let results = client
        .scan()
        .table_name("electricity_failures")
        .filter_expression("#city = :city and #region = :region and #time = :time and #date = :date")
        .expression_attribute_names("#city", "city")
        .expression_attribute_names("#region", "region")
        .expression_attribute_names("#time", "time")
        .expression_attribute_names("#date", "date")
        .expression_attribute_values(":city", city_av)
        .expression_attribute_values(":region", region_av)
        .expression_attribute_values(":time", time_av)
        .expression_attribute_values(":date", date_av)
        .send()
        .await?;

    if let Some(items) = results.items() {
        let item = items.get(0);

        if let Some(attributes) = item {
            println!(
                "{}",
                attributes
                    .get("city")
                    .cloned()
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .to_owned()
            );
            println!(
                "{}",
                attributes
                    .get("date")
                    .cloned()
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .to_owned()
            );
            println!(
                "{}",
                attributes
                    .get("time")
                    .cloned()
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .to_owned()
            );
            println!(
                "{}",
                attributes
                    .get("region")
                    .cloned()
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .to_owned()
            );
            return Ok(Some(
                attributes
                    .get("id")
                    .cloned()
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .to_owned(),
            ));
        }
    }

    Ok(None)
}
