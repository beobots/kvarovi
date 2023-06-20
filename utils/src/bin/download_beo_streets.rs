use anyhow::{anyhow, bail, Context, Result};
use futures::future::try_join_all;
use scraper::{Html, Selector};
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};
use std::sync::OnceLock;

const URL: &str = "https://zis.beograd.gov.rs/ulicebgdout/ulicebgdout.php?page=";
const MAX_PAGES: usize = 283;

static TABLE_SELECTOR: OnceLock<Result<Selector>> = OnceLock::new();

static DATA_SELECTOR: OnceLock<Result<Selector>> = OnceLock::new();

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
    let table_selector = match TABLE_SELECTOR.get_or_init(|| {
        Selector::parse("#ulicebgdoutGrid tbody > tr.pg-row")
            .map_err(|e| anyhow!("failed to initialize table selector: {e}"))
    }) {
        Ok(x) => x,
        Err(e) => bail!(e),
    };

    let data_selector = match DATA_SELECTOR.get_or_init(|| {
        Selector::parse("td:not(:first-child)").map_err(|e| anyhow!("failed to initialize data selector: {e}"))
    }) {
        Ok(x) => x,
        Err(e) => bail!(e),
    };

    let document = Html::parse_document(&body);

    let mut result = Vec::with_capacity(50);
    for row in document.select(table_selector) {
        let mut data_sel = row.select(data_selector);

        let record = data_sel
            .next()
            .and_then(|street_name| {
                data_sel.next().and_then(|old_street_name| {
                    data_sel.next().and_then(|municipality| {
                        data_sel.next().and_then(|settlement| {
                            data_sel.next().and_then(|settlement_part| {
                                data_sel.next().map(|si_list| Record {
                                    street_name: String::from_iter(street_name.text().map(str::trim)),
                                    old_street_name: {
                                        let x = String::from_iter(old_street_name.text().map(str::trim));
                                        match &x[..] {
                                            "" | "NULL" => None,
                                            _ => Some(x),
                                        }
                                    },
                                    municipality: String::from_iter(municipality.text().map(str::trim)),
                                    settlement: String::from_iter(settlement.text().map(str::trim)),
                                    settlement_part: String::from_iter(settlement_part.text().map(str::trim)),
                                    si_list: {
                                        let x = String::from_iter(si_list.text().map(str::trim));
                                        match &x[..] {
                                            "" | "NULL" => None,
                                            _ => Some(x),
                                        }
                                    },
                                })
                            })
                        })
                    })
                })
            })
            .context("failed to extract a row of data")?;

        result.push(record)
    }

    Ok(result)
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
