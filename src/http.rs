use std::time::Duration;

use reqwest::{Client, ClientBuilder};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Http {
    client: Client,
    from: String,
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: String) -> Self {
        Self {
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(30))
                .user_agent(USER_AGENT)
                .build()
                .expect("failed to create reqwest client"),
            from,
        }
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let request = self.client.get(url).header("from", &self.from);
        let response = request.send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}
