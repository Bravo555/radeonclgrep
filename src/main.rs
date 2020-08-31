use chrono::{offset::TimeZone, prelude::*, Date};
use regex::Regex;
use reqwest::{header, Client};
use scraper::{ElementRef, Html, Selector};
use std::env;
use std::error::Error;
use std::fmt::Display;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pattern = Regex::new(&format!(
        "(?i:{})",
        &env::args().nth(1).expect("expected pattern"),
    ))?;
    let current_date = Utc::today();
    let client = http_client()?;

    let responses =
        tokio::spawn(async move { get_month_revisions(&client, &current_date).await.unwrap() })
            .await?;

    for response in responses {
        print_page_hits(&response, &pattern);
    }
    Ok(())
}

fn format_url(year: impl Display, month: impl Display, revision: u32) -> String {
    let base_url = "https://www.amd.com/en/support/kb/release-notes/rn-rad-win";
    format!("{}-{}-{}-{}", base_url, year, month, revision)
}

fn http_client() -> reqwest::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT_ENCODING,
        header::HeaderValue::from_static(""),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-GB,en;q=0.5"),
    );
    Client::builder().default_headers(headers).build()
}

fn print_page_hits(response: &str, pattern: &Regex) {
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
}

async fn get_month_revisions<Tz: TimeZone>(
    client: &Client,
    date: &Date<Tz>,
) -> reqwest::Result<Vec<String>>
where
    <Tz as TimeZone>::Offset: Display,
{
    let mut current_revision = 1;
    let mut revisions = Vec::new();

    loop {
        let version_url = format_url(date.format("%y"), date.month(), current_revision);
        let response = client.get(&version_url).send().await?;

        if !response.status().is_success() {
            break;
        }
        revisions.push(response.text().await?);
        current_revision += 1;
    }

    Ok(revisions)
}
