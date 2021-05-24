use std::time::Duration;

use reqwest::{header, Client, ClientBuilder};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Http {
    client: Client,
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        // TODO: validate from this way
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
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}
