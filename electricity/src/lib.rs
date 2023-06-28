extern crate reqwest;

use addresses::AddressRecord;
use anyhow::{anyhow, Context, Ok, Result};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::NaiveDate;
use db::create_table;
use scraper::Selector;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use uuid::Uuid;

mod addresses;

pub mod db;
pub mod elektrodistribucija_parser;
pub mod time_interval;
pub mod translit;

use elektrodistribucija_parser::{get_content_table_html, get_page_date, get_page_header};

use crate::translit::Translit;

const RAW_DATA_TABLE_NAME: &str = "electricity_failures_raw";
const DATA_TABLE_NAME: &str = "electricity_failures";

static TR_SELECTOR: OnceLock<Selector> = OnceLock::new();
static TD_SELECTOR: OnceLock<Selector> = OnceLock::new();

fn tr_selector() -> &'static Selector {
    TR_SELECTOR.get_or_init(|| Selector::parse("tr").expect("failed to initialize tr selector"))
}

fn td_selector() -> &'static Selector {
    TD_SELECTOR.get_or_init(|| Selector::parse("td").expect("failed to initialize td selector"))
}

#[derive(Debug, Clone)]
pub struct ElectricityFailuresData {
    city: String,
    region: String,
    time: String,
    date: String,
    addresses: addresses::Addresses,
}

impl Display for ElectricityFailuresData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ city: {}, region: {}, time: {}, date: {}, addresses: {} }}",
            self.city, self.region, self.time, self.date, self.addresses
        )
    }
}

#[derive(Debug)]
pub struct ElectricityFailuresRawData {
    id: String,
    date: String,
    url: String,
    html: String,
    hash: String,
    version: i32,
}

impl Display for ElectricityFailuresRawData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ id: {}, date: {}, url: {}, html: {}, hash: {}, version: {} }}",
            self.id, self.date, self.url, self.html, self.hash, self.version
        )
    }
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
    let date = get_page_date(html).and_then(format_date)?;
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
                .and_then(|av| av.as_n().ok())
                .map(|n| n.parse::<i32>().expect("failed to parse version"))
                .context("version is missing")
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

pub async fn parse_and_save_raw_data(client: &Client, id: &str) -> Result<()> {
    let raw_data = find_electricity_failure_raw_data_by_id(client, id).await?;
    let data = parse_raw_data_to_data(&raw_data)?;

    create_table(client, DATA_TABLE_NAME, "id").await?;

    for d in data {
        save_electricity_failure_data(client, &d).await?;
    }

    Ok(())
}

async fn add_electricity_failure_record(
    client: &Client,
    data: &ElectricityFailuresData,
    record: &AddressRecord,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let city_av = AttributeValue::S(data.city.to_owned());
    let region_av = AttributeValue::S(data.region.to_owned());
    let time_av = AttributeValue::S(data.time.to_owned());
    let street_av = AttributeValue::S(record.street.to_owned());
    let date_av = AttributeValue::S(data.date.to_owned());

    let request = client
        .put_item()
        .table_name(DATA_TABLE_NAME)
        .item("id", AttributeValue::S(id))
        .item("city", city_av)
        .item("region", region_av)
        .item("time", time_av)
        .item("street", street_av)
        .item("date", date_av);

    let _ = request.send().await?;

    Ok(())
}

pub async fn save_electricity_failure_data(client: &Client, data: &ElectricityFailuresData) -> Result<()> {
    let addresses = data.addresses.clone();

    for address in addresses.into_iter() {
        add_electricity_failure_record(client, data, &address).await?;
    }

    Ok(())
}

async fn find_electricity_failure_raw_data_by_id(client: &Client, id: &str) -> Result<ElectricityFailuresRawData> {
    let id_av = AttributeValue::S(id.to_owned());

    let results = client
        .scan()
        .table_name(RAW_DATA_TABLE_NAME)
        .filter_expression("#id = :id")
        .expression_attribute_names("#id", "id")
        .expression_attribute_values(":id", id_av)
        .send()
        .await?;

    let mut item = None;

    if let Some(items) = results.items() {
        item = items.get(0);
    }

    if let Some(attributes) = item {
        let id = attributes
            .get("id")
            .and_then(|av| av.as_s().ok())
            .map(ToOwned::to_owned)
            .context("id is missing")?;
        let date = attributes
            .get("date")
            .and_then(|av| av.as_s().ok())
            .map(ToOwned::to_owned)
            .context("date is missing")?;
        let url = attributes
            .get("url")
            .and_then(|av| av.as_s().ok())
            .map(ToOwned::to_owned)
            .context("url is missing")?;
        let html = attributes
            .get("html")
            .and_then(|av| av.as_s().ok())
            .map(ToOwned::to_owned)
            .context("html is missing")?;
        let hash = attributes
            .get("hash")
            .and_then(|av| av.as_s().ok())
            .map(ToOwned::to_owned)
            .context("hash is missing")?;
        let version = attributes
            .get("version")
            .and_then(|av| av.as_n().ok())
            .map(|n| n.parse::<i32>().expect("failed to parse version"))
            .context("version is missing")?;

        return Ok(ElectricityFailuresRawData {
            id,
            date,
            url,
            html,
            hash,
            version,
        });
    }

    Err(anyhow!("Item not found"))
}

fn parse_raw_data_to_data(data: &ElectricityFailuresRawData) -> Result<Vec<ElectricityFailuresData>> {
    let page_html = data.html.to_owned();
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
    let tr_selector = tr_selector();
    let td_selector = td_selector();

    let mut table_rows: Vec<ElectricityFailuresData> = vec![];

    let rows = table.select(tr_selector).collect::<Vec<_>>();
    let heading_row = rows.get(0).ok_or(anyhow!("Heading row is missing"))?;
    let heading_td = heading_row.select(td_selector);

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

    for row in rows.iter().skip(1) {
        let cells = row.select(td_selector).collect::<Vec<_>>();

        let region = cells
            .get(region_index)
            .ok_or(anyhow!("Cell is missing"))?
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<String>();
        let time = cells
            .get(time_index)
            .ok_or(anyhow!("Cell is missing"))?
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<String>();
        let street = cells
            .get(street_index)
            .ok_or(anyhow!("Cell is missing"))?
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<String>();
        let translited_street = street.translit();

        let addresses = addresses::Addresses::parse(&translited_street).map_err(|e| e.to_owned())?;

        table_rows.push(ElectricityFailuresData {
            city: city.to_owned(),
            region,
            time,
            date: format_date(date.to_owned())?,
            addresses,
        });
    }

    Ok(table_rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_page_to_rows() {
        let data = ElectricityFailuresRawData {
            id: String::from("id"),
            date: String::from("01-01-2021"),
            url: String::from("url"),
            html: String::from(
                r#"
                <html>
                    <body>
                        <table>
                            <tbody>
                                <tr>
                                    <td>
                                        <b>Скопје - Центар - 01.01.2021</b>
                                    </td>
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
                                    <td>Бул. Климент Охридски: 43-46</td>
                                </tr>
                            </tbody>
                        </table>
                    </body>
                </html>
            "#,
            ),
            hash: String::from("hash"),
            version: 1,
        };
        let rows = parse_raw_data_to_data(&data).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].city, "Скопје");
        assert_eq!(rows[0].region, "Центар");
        assert_eq!(rows[0].time, "08:00 - 16:00");
        assert_eq!(rows[0].date, "01-01-2021");
    }
}
