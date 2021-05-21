use serde::{Deserialize, Serialize};
use url::Url;

use super::general::format_url_as_filename;
use crate::http::Http;

use super::general::{HuntOutput, Huntable};

#[derive(Debug, Deserialize, Serialize)]
pub struct Utf8 {
    pub url: Url,
}

impl Huntable for Utf8 {
    fn hunt(&self, http_agent: &Http) -> anyhow::Result<HuntOutput> {
        let content = http_agent.get(self.url.as_str())?;
        let filename = format_url_as_filename(&self.url, "txt");
        Ok(HuntOutput { content, filename })
    }
}
