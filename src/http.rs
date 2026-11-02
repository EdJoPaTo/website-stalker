use core::time::Duration;
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::Context as _;
use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use regex::bytes::Regex as BytesRegex;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{ClientBuilder, header};
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

fn extract_html_charset(bytes: &[u8]) -> Option<&'static Encoding> {
    static META_CHARSET_RE: LazyLock<BytesRegex> = LazyLock::new(|| {
        BytesRegex::new(r#"(?i)<meta\s+[^>]*charset=["']?\s*([a-zA-Z0-9._-]+)"#).unwrap()
    });

    if let Some(captures) = META_CHARSET_RE.captures(bytes) {
        if let Some(m) = captures.get(1) {
            return Encoding::for_label(m.as_bytes());
        }
    }
    None
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
) -> reqwest::Result<(Content, ResponseMeta)> {
    let mut builder = ClientBuilder::new()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .timeout(Duration::from_secs(30))
        .user_agent(HeaderValue::from_static(USER_AGENT));
    if http1_only {
        builder = builder.http1_only();
    }
    let request = builder.build()?.get(url).headers(additional_headers);

    let start = Instant::now();
    let response = request.send().await?.error_for_status()?;
    let took = Instant::now().saturating_duration_since(start);

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let extension = content_type.as_deref().and_then(mime2ext::mime2ext);
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

    let bytes = response.bytes().await?;

    let encoding = Encoding::for_bom(&bytes)
        .map(|(enc, _)| enc)
        .or_else(|| extract_html_charset(&bytes))
        .or_else(|| {
            content_type.as_deref().and_then(|ct| {
                ct.split(';')
                    .find_map(|param| {
                        let param = param.trim();
                        if param.to_lowercase().starts_with("charset=") {
                            let charset = param["charset=".len()..].trim_matches('"');
                            Encoding::for_label(charset.as_bytes())
                        } else {
                            None
                        }
                    })
            })
        })
        .unwrap_or_else(|| {
            let mut detector = EncodingDetector::new();
            detector.feed(&bytes, true);
            detector.guess(None, true)
        });

    let (cow, _, _) = encoding.decode(&bytes);
    let text = cow.replace("\r\n", "\n").replace('\r', "\n");

    let content = Content { extension, text };
    Ok((content, meta))
}

pub fn validate_from(from: &str) -> anyhow::Result<()> {
    let value = HeaderValue::from_str(from)?;
    let value = value.to_str().context("contains non ASCII characters")?;
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
