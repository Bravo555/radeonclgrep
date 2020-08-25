use chrono::prelude::*;
use regex::Regex;
use reqwest::{header, Client};
use scraper::{ElementRef, Html, Selector};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pattern = Regex::new(&env::args().nth(1).expect("expected pattern")).unwrap();

    let base_url = "https://www.amd.com/en/support/kb/release-notes/rn-rad-win-";
    let current_date = Utc::now().date().naive_utc();
    let version_url = format!(
        "{}{}-{}-{}",
        base_url,
        current_date.format("%y"),
        current_date.month(),
        2
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT_ENCODING,
        header::HeaderValue::from_static(""),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-GB,en;q=0.5"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    let response = client.get(&version_url).send().await?.text().await?;

    let doc = Html::parse_document(&response);
    let sel = Selector::parse(".field--name-body > ul").unwrap();
    let lists = doc.select(&sel).take(4);
    let hits = lists.flat_map(|n| {
        n.children()
            .filter_map(ElementRef::wrap)
            .map(|er| er.inner_html())
            .filter(|s| pattern.is_match(&s))
            .map(|li| html2text::from_read(li.as_bytes(), 80))
    });

    for hit in hits {
        println!("{}", hit);
    }
    Ok(())
}
