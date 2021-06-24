use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Response;
use crate::regex_replacer::RegexReplacer;

pub use self::css_selector::CssSelector;

use super::url_filename;

mod css_selector;
mod prettify;

#[derive(Debug, Deserialize, Serialize)]
pub struct Html {
    pub url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<CssSelector>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regex_replacers: Vec<RegexReplacer>,
}

impl Html {
    pub fn get_filename(&self) -> String {
        url_filename::format(&self.url, "html")
    }

    pub async fn stalk(&self, response: Response) -> anyhow::Result<String> {
        let content = response.text().await?;

        #[allow(clippy::option_if_let_else)]
        let content = if let Some(selector) = &self.css_selector {
            let selected = selector.select(&content);
            if selected.is_empty() {
                return Err(anyhow::anyhow!(
                    "css_selector ({}) selected nothing",
                    selector
                ));
            }
            selected.join("\n")
        } else {
            content
        };

        let prettified = prettify::prettify(&content)?;

        let mut replaced = prettified;
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
