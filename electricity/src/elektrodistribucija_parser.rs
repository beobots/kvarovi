use scraper::{Html, Selector, ElementRef};

pub fn get_page_header(page_html: &str) -> String {
    let document = Html::parse_document(&page_html);
    let table_selector = Selector::parse("table").unwrap();
    let tables = document.select(&table_selector).collect::<Vec<ElementRef>>();

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

    header
}

pub fn get_content_table_html(page_html: &str) -> String {
    let document = Html::parse_document(&page_html);
    let table_selector = Selector::parse("table").unwrap();
    let tables = document.select(&table_selector).collect::<Vec<ElementRef>>();

    tables.get(1).unwrap().html()
}

pub fn get_page_date(page_html: &str) -> String {
    let header = get_page_header(page_html);
    let date = header
        .split(' ')
        .last()
        .unwrap()
        .to_owned()
        .split('.')
        .filter(|&str| !str.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    date
}