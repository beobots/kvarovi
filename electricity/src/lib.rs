extern crate reqwest;

use anyhow::{anyhow, Ok, Result};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::NaiveDate;
use db::create_table;
use scraper::Selector;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

mod addresses;

pub mod db;
pub mod elektrodistribucija_parser;
pub mod time_interval;
pub mod translit;

use elektrodistribucija_parser::{get_content_table_html, get_page_date, get_page_header};

const RAW_DATA_TABLE_NAME: &str = "electricity_failures_raw";

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

pub async fn collect_data(db_client: &Client, pages: &[String]) -> Result<()> {
    let requests = pages.iter().map(|page| {
        let p = page.clone();
        tokio::task::spawn_blocking(move || {
            tokio::task::spawn(async move {
                let html = fetch_page(&p).await;

                (p.clone(), html)
            })
        })
    });

    let results = futures::future::join_all(requests).await;

    let _ = create_table(db_client, RAW_DATA_TABLE_NAME, "id").await;

    for result in results {
        let (page, html) = result.unwrap().await?;

        let _ = add_electricity_failure_raw_item(db_client, &html?, &page).await;
    }

    Ok(())
}

fn format_date(date: String) -> Result<String> {
    let mut naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d");

    if naive_date.is_err() {
        naive_date = NaiveDate::parse_from_str(&date, "%d-%m-%Y");
    }

    Ok(naive_date?.format("%d-%m-%Y").to_string())
}

async fn add_electricity_failure_raw_item(client: &Client, html: &str, page: &str) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let date = format_date(get_page_date(html))?;
    let page = page.to_owned();
    let hash = {
        let mut hasher = DefaultHasher::new();
        html.hash(&mut hasher);
        hasher.finish().to_string()
    };

    let (last_version, last_version_hash) =
        find_last_electricity_failure_raw_version(client, page.to_owned(), date.to_owned()).await?;

    if let Some(last_version_hash) = last_version_hash {
        if last_version_hash == hash {
            return Ok(());
        }
    }

    let id_av = AttributeValue::S(id);
    let date_av = AttributeValue::S(date);
    let page_av = AttributeValue::S(page);
    let html_av = AttributeValue::S(html.to_owned());
    let hash_av = AttributeValue::S(hash);
    let version_av = AttributeValue::N((last_version + 1).to_string());

    let request = client
        .put_item()
        .table_name(RAW_DATA_TABLE_NAME)
        .item("id", id_av)
        .item("date", date_av)
        .item("url", page_av)
        .item("html", html_av)
        .item("hash", hash_av)
        .item("version", version_av);

    let _ = request.send().await?;

    Ok(())
}

async fn find_last_electricity_failure_raw_version(
    client: &Client,
    url: String,
    date: String,
) -> Result<(i32, Option<String>)> {
    let url_av = AttributeValue::S(url);
    let date_av = AttributeValue::S(date);

    let results = client
        .scan()
        .table_name(RAW_DATA_TABLE_NAME)
        .filter_expression("#url = :url and #date = :date")
        .expression_attribute_names("#url", "url")
        .expression_attribute_names("#date", "date")
        .expression_attribute_values(":url", url_av)
        .expression_attribute_values(":date", date_av)
        .send()
        .await?;

    let mut last_version = 0;
    let mut last_version_hash = None;

    if let Some(items) = results.items() {
        items.iter().for_each(|item| {
            let version = item
                .get("version")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i32>()
                .unwrap();
            let hash = item.get("hash").unwrap().as_s().unwrap().to_owned();
            if version > last_version {
                last_version = version;
                last_version_hash = Some(hash);
            }
        });
    }

    Ok((last_version, last_version_hash))
}

// TODO theses functions are for parsing html to structured data

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

#[allow(dead_code)]
async fn add_electricity_failure_item(client: &Client, item: &TableRowData) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let city_av = AttributeValue::S(item.city.to_owned());
    let region_av = AttributeValue::S(item.region.to_owned());
    let time_av = AttributeValue::S(item.time.to_owned());
    let street_av = AttributeValue::S(item.street.to_owned());
    let date_av = AttributeValue::S(item.date.to_owned());

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
fn parse_page_to_rows(page_html: String) -> Result<Vec<TableRowData>> {
    let header: String = get_page_header(&page_html);
    let date = header
        .split(' ')
        .last()
        .ok_or(anyhow!("Cell is missing"))?
        .to_owned()
        .split('.')
        .filter(|&str| !str.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let city = header
        .split(" - ")
        .collect::<Vec<_>>()
        .first()
        .ok_or(anyhow!("Cell is missing"))?
        .to_string();

    let table = get_content_table_html(&page_html);
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut table_rows: Vec<TableRowData> = vec![];

    let rows = table.select(&tr_selector).collect::<Vec<_>>();
    let heading_row = rows.get(0).ok_or(anyhow!("Heading row is missing"))?;
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
            date: format_date(date.clone())?,
        })
    }

    Ok(table_rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_page_to_rows() {
        let page_html = r#"
            <html>
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <table>
                        <tbody>
                            <tr>
                                <td><b>Скопје - Центар - 01.01.2021</b></td>
                            </tr>
                        </tbody>
                    </table>
                    <table>
                        <tbody>
                            <tr>
                                <td>Општина</td>
                                <td>Време</td>
                                <td>Улице</td>
                            </tr>
                            <tr>
                                <td>Центар</td>
                                <td>08:00 - 16:00</td>
                                <td>Бул. Климент Охридски</td>
                            </tr>
                        </tbody>
                    </table>
                </body>
            </html>
        "#;

        let rows = parse_page_to_rows(page_html.to_owned()).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].city, "Скопје");
        assert_eq!(rows[0].region, "Центар");
        assert_eq!(rows[0].street, "Бул. Климент Охридски");
        assert_eq!(rows[0].time, "08:00 - 16:00");
        assert_eq!(rows[0].date, "01-01-2021");
    }
}
