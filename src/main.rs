use reqwest::{header, Client};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    let response = client
        .get("https://www.amd.com/en/support/kb/release-notes/rn-rad-win-20-8-2")
        .send()
        .await?
        .text()
        .await?;
    println!("{:?}", &response);
    Ok(())
}
