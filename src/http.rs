use core::time::Duration;
use std::net::SocketAddr;
use std::time::Instant;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, ClientBuilder};
use url::Url;

use crate::editor::Content;

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct ResponseMeta {
    pub http_version: reqwest::Version,
    pub ip_version: IpVersion,
    pub took: Duration,
    /// Get the final `Url` of this `Response`.
    pub url: Url,
}

#[derive(Debug)]
pub enum IpVersion {
    IPv4,
    IPv6,
    None,
}

impl core::fmt::Display for IpVersion {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, fmt)
    }
}

/// HTTP GET Request
///
/// FROM provides an email address for the target host to be contacted in case of problems.
/// See [HTTP From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
pub async fn get(
    url: &str,
    additional_headers: HeaderMap,
    accept_invalid_certs: bool,
    http1_only: bool,
) -> anyhow::Result<(Content, ResponseMeta)> {
    let mut builder = ClientBuilder::new()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT);
    if http1_only {
        builder = builder.http1_only();
    }
    let request = builder.build()?.get(url).headers(additional_headers);

    let start = Instant::now();
    let response = request.send().await?.error_for_status()?;
    let took = Instant::now().saturating_duration_since(start);

    let extension = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(mime2ext::mime2ext);
    let ip_version = match response.remote_addr() {
        Some(SocketAddr::V4(_)) => IpVersion::IPv4,
        Some(SocketAddr::V6(_)) => IpVersion::IPv6,
        None => IpVersion::None,
    };
    let meta = ResponseMeta {
        http_version: response.version(),
        ip_version,
        took,
        url: response.url().clone(),
    };
    let text = response.text().await?;
    let content = Content { extension, text };
    Ok((content, meta))
}

pub fn validate_from(from: &str) -> anyhow::Result<()> {
    let value = HeaderValue::from_str(from)?;
    let value = value
        .to_str()
        .map_err(|err| anyhow::anyhow!("from contains non ASCII characters {err}"))?;
    if !value.contains('@') || !value.contains('.') {
        anyhow::bail!("doesnt look like an email address: {from}");
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
#[should_panic = "ASCII char"]
fn from_is_no_ascii() {
    validate_from("föo@bär.de").unwrap();
}
