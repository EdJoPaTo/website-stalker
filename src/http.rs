use std::time::Duration;

use ureq::{Agent, AgentBuilder, Request};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Http {
    agent: Agent,
    from: String,
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: String) -> Self {
        Self {
            agent: AgentBuilder::new()
                .timeout(Duration::from_secs(30))
                .user_agent(USER_AGENT)
                .build(),
            from,
        }
    }

    fn get_with_headers(&self, url: &str) -> Request {
        self.agent.get(url).set("from", &self.from)
    }

    pub fn get(&self, url: &str) -> anyhow::Result<String> {
        let text = self.get_with_headers(url).call()?.into_string()?;
        Ok(text)
    }
}
