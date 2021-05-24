use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;
use crate::regex_replacer::RegexReplacer;

use super::url_filename;

#[derive(Debug, Deserialize, Serialize)]
pub struct Utf8 {
    pub url: Url,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regex_replacers: Vec<RegexReplacer>,
}

impl Utf8 {
    pub fn get_filename(&self) -> String {
        url_filename::format(&self.url, "txt")
    }

    pub async fn hunt(&self, http_agent: &Http) -> anyhow::Result<String> {
        let content = http_agent.get(self.url.as_str()).await?;

        let mut replaced = content;
        for replacer in &self.regex_replacers {
            replaced = replacer.replace_all(&replaced)?.to_string();
        }

        Ok(replaced)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        for rp in &self.regex_replacers {
            rp.is_valid()?;
        }

        Ok(())
    }
}
