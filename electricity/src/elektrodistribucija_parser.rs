use std::sync::OnceLock;

use anyhow::{Context, Ok, Result};
use scraper::{ElementRef, Html, Selector};

static TABLE_SELECTOR: OnceLock<Selector> = OnceLock::new();

static HEADER_SELECTOR: OnceLock<Selector> = OnceLock::new();

fn table_selector() -> &'static Selector {
    TABLE_SELECTOR.get_or_init(|| Selector::parse("table").expect("table selector initialized"))
}

fn header_selector() -> &'static Selector {
    HEADER_SELECTOR.get_or_init(|| Selector::parse("tbody > tr > td > b").expect("header selector initialized"))
}

pub fn get_page_header(page_html: &str) -> String {
    let header_table = get_header_table_html(page_html);
    String::from_iter(
        header_table
            .select(header_selector())
            .next()
            .into_iter()
            .flat_map(|it| it.text())
            .map(str::trim),
    )
}

pub fn get_header_table_html(page_html: &str) -> Html {
    let document = Html::parse_document(page_html);
    let table_selector = table_selector();
    let tables = document.select(table_selector).collect::<Vec<ElementRef>>();

    Html::parse_fragment(&tables.get(0).unwrap().html())
}

pub fn get_content_table_html(page_html: &str) -> Html {
    let document = Html::parse_document(page_html);
    let table_selector = table_selector();
    let tables = document.select(table_selector).collect::<Vec<ElementRef>>();

    Html::parse_fragment(&tables.get(1).unwrap().html())
}

pub fn get_page_date(page_html: &str) -> Result<String> {
    let header = get_page_header(page_html);
    let date = header
        .split(':')
        .last()
        .map(str::trim)
        .map(ToOwned::to_owned)
        .context("failed to extract date from header")?;

    Ok(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PAGE_HTML: &str = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <table>
                    <tbody>
                        <tr>
                            <td>
                                <b>БЕОГРАД - Планирана искључења за датум: 2021-01-01</b>
                            </td>
                        </tr>
                    </tbody>
                </table>
                <table>
                    <tbody>
                        <tr>
                            <td>
                                <b>Општина</b>
                            </td>
                            <td>
                                <b>Време</b>
                            </td>
                            <td>
                                <b>Улице</b>
                            </td>
                        </tr>
                        <tr>
                            <td>
                                <b>Општина 1</b>
                            </td>
                            <td>
                                <b>Време 1</b>
                            </td>
                            <td>
                                <b>Улица 1</b>
                            </td>
                        </tr>
                        <tr>
                            <td>
                                <b>Општина 2</b>
                            </td>
                            <td>
                                <b>Време 2</b>
                            </td>
                            <td>
                                <b>Улица 2</b>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </body>
        </html>
    "#;

    #[test]
    fn test_get_page_date_extracts_date() {
        let date = get_page_date(TEST_PAGE_HTML).unwrap();

        assert_eq!(date, "2021-01-01");
    }

    #[test]
    fn test_get_page_header_extracts_header() {
        let header = get_page_header(TEST_PAGE_HTML);

        assert_eq!(header, "БЕОГРАД - Планирана искључења за датум: 2021-01-01");
    }

    #[test]
    fn test_get_header_table_html_extracts_header_table() {
        let header_table = get_header_table_html(TEST_PAGE_HTML);

        assert_eq!(header_table.html(), "<html><table>\n                    <tbody>\n                        <tr>\n                            <td>\n                                <b>БЕОГРАД - Планирана искључења за датум: 2021-01-01</b>\n                            </td>\n                        </tr>\n                    </tbody>\n                </table></html>");
    }

    #[test]
    fn test_get_content_table_html_extracts_content_table() {
        let content_table = get_content_table_html(TEST_PAGE_HTML);

        assert_eq!(content_table.html(), "<html><table>\n                    <tbody>\n                        <tr>\n                            <td>\n                                <b>Општина</b>\n                            </td>\n                            <td>\n                                <b>Време</b>\n                            </td>\n                            <td>\n                                <b>Улице</b>\n                            </td>\n                        </tr>\n                        <tr>\n                            <td>\n                                <b>Општина 1</b>\n                            </td>\n                            <td>\n                                <b>Време 1</b>\n                            </td>\n                            <td>\n                                <b>Улица 1</b>\n                            </td>\n                        </tr>\n                        <tr>\n                            <td>\n                                <b>Општина 2</b>\n                            </td>\n                            <td>\n                                <b>Време 2</b>\n                            </td>\n                            <td>\n                                <b>Улица 2</b>\n                            </td>\n                        </tr>\n                    </tbody>\n                </table></html>");
    }
}
