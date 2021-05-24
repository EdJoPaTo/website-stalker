use std::time::Duration;

use reqwest::{header, Client, ClientBuilder};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

#[derive(Clone)]
pub struct Http {
    client: Client,
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::FROM, header::HeaderValue::from_str(from).unwrap());

        Self {
            client: ClientBuilder::new()
                .default_headers(headers)
                .timeout(Duration::from_secs(30))
                .user_agent(USER_AGENT)
                .build()
                .expect("failed to create reqwest client"),
        }
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let response = self.client.get(url).send().await?.error_for_status()?;
        let text = response.text().await?;
        Ok(text)
    }
}

pub fn validate_from(from: &str) -> anyhow::Result<()> {
    let value = header::HeaderValue::from_str(from)?;
    let value = value.to_str()?;
    if !value.contains('@') || !value.contains('.') {
        return Err(anyhow::anyhow!(
            "doesnt look like an email address: {}",
            from
        ));
    }

    Ok(())
}

#[test]
fn from_is_email() {
    let result = validate_from("foo@bar.de");
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn from_is_no_email() {
    let result = validate_from("bla.de");
    println!("{:?}", result);
    assert!(result.is_err());
}

#[test]
fn from_is_no_ascii() {
    let result = validate_from("f\u{f6}o@b\u{e4}r.de");
    println!("{:?}", result);
    assert!(result.is_err());
}
