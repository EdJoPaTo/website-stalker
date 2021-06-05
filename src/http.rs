use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, ClientBuilder, StatusCode};

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

pub struct Response {
    response: reqwest::Response,
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

    pub async fn get<T>(&self, url: &str, last_change: Option<T>) -> anyhow::Result<Response>
    where
        T: Into<DateTime<Utc>>,
    {
        let mut headers = HeaderMap::new();

        if let Some(last_change) = last_change {
            let dt: DateTime<Utc> = last_change.into();
            let last_change = dt.to_rfc2822().replace("+0000", "GMT");
            headers.append(
                header::IF_MODIFIED_SINCE,
                HeaderValue::from_str(&last_change)?,
            );
        }

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;
        Ok(Response { response })
    }
}

impl Response {
    pub fn is_not_modified(&self) -> bool {
        self.response.status() == StatusCode::NOT_MODIFIED
    }

    pub async fn text(self) -> anyhow::Result<String> {
        let text = self.response.text().await?;
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
