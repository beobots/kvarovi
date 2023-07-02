use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Selector};
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};
use std::sync::OnceLock;

const URL: &str = "https://zis.beograd.gov.rs/ulicebgdout/ulicebgdout.php?page=";
const MAX_PAGES: usize = 283;

static TABLE_SELECTOR: OnceLock<Selector> = OnceLock::new();

static DATA_SELECTOR: OnceLock<Selector> = OnceLock::new();

fn table_selector() -> &'static Selector {
    TABLE_SELECTOR
        .get_or_init(|| Selector::parse("#ulicebgdoutGrid tbody > tr.pg-row").expect("initialize CSS selector"))
}

fn data_selector() -> &'static Selector {
    DATA_SELECTOR.get_or_init(|| Selector::parse("td:not(:first-child)").expect("initialize CSS selector"))
}

#[cfg_attr(test, derive(Debug, PartialEq))]
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

fn extract_dataset(body: impl AsRef<str>) -> Result<Vec<Record>> {
    let table_selector = table_selector();
    let data_selector = data_selector();

    let document = Html::parse_document(body.as_ref());

    let mut result = Vec::with_capacity(50);
    for row in document.select(table_selector) {
        let data_sel = row.select(data_selector);
        let record = extract_record(data_sel).context("failed to extract a row of data")?;
        result.push(record)
    }

    Ok(result)
}

fn extract_record<'a>(mut data_sel: impl Iterator<Item = ElementRef<'a>>) -> Option<Record> {
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
/// Pages are downloaded one after another, because the the web server cannot
/// handle many connection simultaneously.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_selector() {
        let sel = table_selector();
        let tests = [
            (
                r#"<html>
            <table id="ulicebgdoutGrid">
                <thead>
                    <tr><td>wrong</td></tr>
                </thead>
                <tbody>
                    <tr><td>wrong</td></tr>
                    <tr class="pg-row"><td>data</td></tr>
                </tbody>
            </table>
        </html>"#,
                Some("data"),
            ),
            (
                r#"<html>
            <table id="ulicebgdoutGrid">
                <thead>
                    <tr><td>wrong</td></tr>
                </thead>
                <tbody>
                    <tr><td>wrong</td></tr>
                </tbody>
            </table>
        </html>"#,
                None,
            ),
        ];

        for (html, maybe_expected_value) in tests {
            let body = Html::parse_document(html);
            let maybe_data = body.select(sel).next();
            if let Some(expected_value) = maybe_expected_value {
                let data = maybe_data.unwrap();
                let text = extract_element_text(data);
                assert_eq!(text, expected_value);
            } else {
                assert!(maybe_data.is_none())
            }
        }
    }

    #[test]
    fn test_extract_dataset() {
        let html = r#"<html><table id="ulicebgdoutGrid">
            <tbody>
            <tr class="pg-row">
                <td> 1 </td>
                <td> street_name1 </td>
                <td> old_street_name1 </td>
                <td> municipality1 </td>
                <td> settlement1 </td>
                <td> settlement_part1 </td>
                <td> si_list1 </td>
            </tr>
            <tr class="pg-row">
                <td> 2 </td>
                <td> street_name2 </td>
                <td> old_street_name2 </td>
                <td> municipality2 </td>
                <td> settlement2 </td>
                <td> settlement_part2 </td>
                <td> si_list2 </td>
            </tr>
            </tbody>
        </table></html>"#;
        let result = extract_dataset(html).unwrap();
        assert_eq!(
            result,
            vec![
                Record {
                    street_name: "street_name1".to_string(),
                    old_street_name: Some("old_street_name1".to_string()),
                    municipality: "municipality1".to_string(),
                    settlement: "settlement1".to_string(),
                    settlement_part: "settlement_part1".to_string(),
                    si_list: Some("si_list1".to_string()),
                },
                Record {
                    street_name: "street_name2".to_string(),
                    old_street_name: Some("old_street_name2".to_string()),
                    municipality: "municipality2".to_string(),
                    settlement: "settlement2".to_string(),
                    settlement_part: "settlement_part2".to_string(),
                    si_list: Some("si_list2".to_string()),
                }
            ]
        )
    }

    #[test]
    fn test_extract_element() {
        let html = r"<table><tr><td> text </td></tr></table>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("td").unwrap();
        let td = fragment.select(&sel).next().unwrap();

        assert_eq!(extract_element_text(td), "text");
    }

    #[test]
    fn test_extract_nullable_element() {
        let sel = Selector::parse("td").unwrap();
        let tests = [
            (r"<table><tr><td>  </td></tr></table>", None),
            (r"<table><tr><td> NULL </td></tr></table>", None),
            (
                r"<table><tr><td> text </td></tr></table>",
                Some("text".to_string()),
            ),
        ];

        for (html, expected_value) in tests {
            let fragment = Html::parse_fragment(html);
            let td = fragment.select(&sel).next().unwrap();
            assert_eq!(extract_nullable_element_text(td), expected_value);
        }
    }

    #[test]
    fn test_extract_record() {
        let sel = data_selector();

        let tests = [
            (
                r"<table>
            <tr>
                <td> 1 </td>
                <td> street_name </td>
                <td> old_street_name </td>
                <td> municipality </td>
                <td> settlement </td>
                <td> settlement_part </td>
                <td> si_list </td>
            </tr>
        </table>",
                Some(Record {
                    street_name: "street_name".to_string(),
                    old_street_name: Some("old_street_name".to_string()),
                    municipality: "municipality".to_string(),
                    settlement: "settlement".to_string(),
                    settlement_part: "settlement_part".to_string(),
                    si_list: Some("si_list".to_string()),
                }),
            ),
            (
                r"<table>
            <tr>
                <td> 1 </td>
                <td> street_name </td>
                <td>  </td>
                <td> municipality </td>
                <td> settlement </td>
                <td> settlement_part </td>
                <td> NULL </td>
            </tr>
        </table>",
                Some(Record {
                    street_name: "street_name".to_string(),
                    old_street_name: None,
                    municipality: "municipality".to_string(),
                    settlement: "settlement".to_string(),
                    settlement_part: "settlement_part".to_string(),
                    si_list: None,
                }),
            ),
            (
                r"<table>
            <tr>
                <td> 1 </td>
                <td> street_name </td>
                <td>  </td>
                <td> municipality </td>
                <td> settlement </td>
                <td> settlement_part </td>
            </tr>
        </table>",
                None,
            ),
        ];

        for (html, expected_value) in tests {
            let document = Html::parse_fragment(html);
            let tds = document.select(sel);
            let result = extract_record(tds);

            assert_eq!(result, expected_value)
        }
    }
}
