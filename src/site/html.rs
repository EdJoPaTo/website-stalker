use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

use super::url_filename;
use super::Huntable;

mod prettify;

#[derive(Debug, Deserialize, Serialize)]
pub struct Html {
    pub url: Url,
}

impl Huntable for Html {
    fn get_filename(&self) -> String {
        url_filename::format(&self.url, "html")
    }

    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String> {
        let content = http_agent.get(self.url.as_str())?;
        let result = prettify::prettify(&content)?;
        Ok(result)
    }
}
