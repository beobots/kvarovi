use anyhow::{Context, Result};
use scraper::element_ref::Select;
use scraper::{ElementRef, Html, Selector};
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};
use std::sync::OnceLock;

const URL: &str = "https://zis.beograd.gov.rs/ulicebgdout/ulicebgdout.php?page=";
const MAX_PAGES: usize = 283;

static TABLE_SELECTOR: OnceLock<Selector> = OnceLock::new();

static DATA_SELECTOR: OnceLock<Selector> = OnceLock::new();

#[derive(Debug)]
struct Record {
    street_name: String,
    old_street_name: Option<String>,
    municipality: String,
    settlement: String,
    settlement_part: String,
    si_list: Option<String>,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{}",
            self.street_name,
            match self.old_street_name {
                Some(ref x) => x.as_str(),
                None => "",
            },
            self.municipality,
            self.settlement,
            self.settlement_part,
            match self.si_list {
                Some(ref x) => x.as_str(),
                None => "",
            }
        )
    }
}

fn extract_dataset(body: String) -> Result<Vec<Record>> {
    let table_selector = TABLE_SELECTOR
        .get_or_init(|| Selector::parse("#ulicebgdoutGrid tbody > tr.pg-row").expect("initialize CSS selector"));

    let data_selector =
        DATA_SELECTOR.get_or_init(|| Selector::parse("td:not(:first-child)").expect("initialize CSS selector"));

    let document = Html::parse_document(&body);

    let mut result = Vec::with_capacity(50);
    for row in document.select(table_selector) {
        let data_sel = row.select(data_selector);
        let record = extract_record(data_sel).context("failed to extract a row of data")?;
        result.push(record)
    }

    Ok(result)
}

fn extract_record(mut data_sel: Select) -> Option<Record> {
    let street_name = data_sel.next().map(extract_element_text)?;
    let old_street_name = data_sel.next().map(extract_nullable_element_text)?;
    let municipality = data_sel.next().map(extract_element_text)?;
    let settlement = data_sel.next().map(extract_element_text)?;
    let settlement_part = data_sel.next().map(extract_element_text)?;
    let si_list = data_sel.next().map(extract_nullable_element_text)?;

    Some(Record {
        street_name,
        old_street_name,
        municipality,
        settlement,
        settlement_part,
        si_list,
    })
}

fn extract_nullable_element_text(value: ElementRef) -> Option<String> {
    let x = extract_element_text(value);
    match &x[..] {
        "" | "NULL" => None,
        _ => Some(x),
    }
}

fn extract_element_text(value: ElementRef) -> String {
    String::from_iter(value.text().map(str::trim))
}

async fn download_page(page: usize) -> Result<Vec<Record>> {
    let body = reqwest::get(format!("{URL}{page}")).await?.text().await?;
    tokio::task::spawn_blocking(move || extract_dataset(body)).await?
}

/// You can run the program as follows:
/// ```bash
/// download_beo_streets | tee download.csv && sort download.csv | uniq | tr '[:upper:]' '[:lower:]' > beograd_streets.csv
/// ```
///
/// Pages are downloaded one after another, because the the web server cannot handle many connection simultaneously.
#[tokio::main]
async fn main() -> Result<()> {
    let mut cout = stdout().lock();
    for page in 1..=MAX_PAGES {
        let dataset = download_page(page).await?;
        for record in dataset {
            writeln!(cout, "{record}")?;
        }
    }
    Ok(())
}
