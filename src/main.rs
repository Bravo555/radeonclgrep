use chrono::{prelude::*, NaiveDate};
use regex::Regex;
use reqwest::{header, Client};
use scraper::{ElementRef, Html, Selector};
use std::env;
use std::error::Error;
use std::fmt::Display;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let DATE_THRESHOLD: NaiveDate = NaiveDate::from_ymd(2020, 1, 1);

    let current_date = Utc::today().naive_utc();
    let months = {
        let mut current = current_date.clone();
        let mut months = Vec::new();
        while current > DATE_THRESHOLD {
            months.push(current.clone());
            current = current
                .with_month(current.month() - 1)
                .or(current
                    .with_year(current.year() - 1)
                    .expect("previous month creation failed")
                    .with_month(12))
                .expect("previous month construction failed");
        }
        months
    };
    println!("{:#?}", months);

    let pattern = Regex::new(&format!(
        "(?i:{})",
        &env::args().nth(1).expect("expected pattern"),
    ))?;
    let client = http_client()?;

    let responses: Vec<_> = months
        .into_iter()
        // we use a helper map here because we cant clone http client in a move block
        .map(|date| (date, client.clone()))
        .map(|(date, client)| {
            tokio::spawn(async move { get_month_revisions(&client, &date).await.unwrap() })
        })
        .collect();

    for response in responses {
        for revision in response.await.unwrap() {
            print_page_hits(&revision, &pattern);
        }
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

async fn get_month_revisions(client: &Client, date: &NaiveDate) -> reqwest::Result<Vec<String>> {
    let mut current_revision = 1;
    let mut revisions = Vec::new();

    loop {
        let version_url = format_url(date.format("%y"), date.month(), current_revision);
        println!("trying {}", version_url);
        let response = client.get(&version_url).send().await?;

        if !response.status().is_success() {
            break;
        }
        revisions.push(response.text().await?);
        current_revision += 1;
    }

    Ok(revisions)
}
