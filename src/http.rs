use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, ClientBuilder, StatusCode};
use url::Url;

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

#[derive(Debug)]
pub enum IpVersion {
    IPv4,
    IPv6,
    None,
}

impl std::fmt::Display for IpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(header::FROM, HeaderValue::from_str(from).unwrap());

        Self {
            client: ClientBuilder::new()
                .default_headers(headers)
                .timeout(Duration::from_secs(30))
                .user_agent(USER_AGENT)
                .build()
                .expect("failed to create reqwest client"),
        }
    }

    pub async fn get(&self, url: &str, last_known_etag: Option<&str>) -> anyhow::Result<Response> {
        let mut headers = HeaderMap::new();
        if let Some(etag) = last_known_etag {
            headers.append(header::IF_NONE_MATCH, HeaderValue::from_str(etag)?);
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

    /// Get the final `Url` of this `Response`.
    pub fn url(&self) -> &Url {
        self.response.url()
    }

    pub fn ip_version(&self) -> IpVersion {
        match self.response.remote_addr() {
            Some(a) => {
                if a.is_ipv6() {
                    IpVersion::IPv6
                } else if a.is_ipv4() {
                    IpVersion::IPv4
                } else {
                    IpVersion::None
                }
            }
            None => IpVersion::None,
        }
    }
}

pub fn validate_from(from: &str) -> anyhow::Result<()> {
    let value = HeaderValue::from_str(from)?;
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
    validate_from("foo@bar.de").unwrap();
}

#[test]
#[should_panic = "doesnt look like an email address"]
fn from_is_no_email() {
    validate_from("bla.de").unwrap();
}

#[test]
#[should_panic]
fn from_is_no_ascii() {
    validate_from("f\u{f6}o@b\u{e4}r.de").unwrap();
}
