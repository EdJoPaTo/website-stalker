use std::time::{Duration, Instant};

use reqwest::header::HeaderValue;
use reqwest::{header, ClientBuilder, StatusCode};
use url::Url;

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Response {
    response: reqwest::Response,
    took: Duration,
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

/// HTTP GET Request
///
/// FROM provides an email address for the target host to be contacted in case of problems.
/// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
pub async fn get(url: &str, from: &str, accept_invalid_certs: bool) -> anyhow::Result<Response> {
    let request = ClientBuilder::new()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()?
        .get(url)
        .header(header::FROM, HeaderValue::from_str(from)?);

    let start = Instant::now();
    let response = request.send().await?.error_for_status()?;
    let took = Instant::now().saturating_duration_since(start);

    Ok(Response { response, took })
}

impl Response {
    pub const fn took(&self) -> Duration {
        self.took
    }

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
