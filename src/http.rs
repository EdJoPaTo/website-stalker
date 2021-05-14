use ureq::{Agent, AgentBuilder, Request};

const USER_AGENT: &str = concat!(
    "website-stalker/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Http {
    agent: Agent,
    user_agent: String,
    from: String,
}

impl Http {
    /// Create an http agent with an email address to be contacted in case of problems.
    ///
    /// See [http From header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From)
    pub fn new(from: String) -> Self {
        Self {
            agent: AgentBuilder::new().build(),
            user_agent: USER_AGENT.to_string(),
            from,
        }
    }

    pub fn set_user_agent(&mut self, user_agent: String) {
        self.user_agent = user_agent;
    }

    fn get_with_headers(&self, url: &str) -> Request {
        self.agent
            .get(url)
            .set("user-agent", &self.user_agent)
            .set("from", &self.from)
    }

    pub fn get(&self, url: &str) -> anyhow::Result<String> {
        let text = self.get_with_headers(url).call()?.into_string()?;
        Ok(text)
    }
}
