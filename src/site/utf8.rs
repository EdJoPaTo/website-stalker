use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

use super::{format_url_as_filename, Huntable};

#[derive(Debug, Deserialize, Serialize)]
pub struct Utf8 {
    pub url: Url,
}

impl Huntable for Utf8 {
    fn get_filename(&self) -> String {
        format_url_as_filename(&self.url, "txt")
    }

    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String> {
        http_agent.get(self.url.as_str())
    }
}
