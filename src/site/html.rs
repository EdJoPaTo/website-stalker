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

    // TODO: remove migration
    #[serde(default, skip_serializing)]
    pub css_selector: Option<String>,

    // TODO: allow for one (String) or many (array)?
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub css_selectors: Vec<CssSelector>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regex_replacers: Vec<RegexReplacer>,
}

impl Html {
    pub fn get_filename(&self) -> String {
        url_filename::format(&self.url, "html")
    }

    pub async fn stalk(&self, response: Response) -> anyhow::Result<String> {
        let mut content = response.text().await?;

        for selector in &self.css_selectors {
            content = selector.apply(&content)?;
        }

        let prettified = prettify::prettify(&content)?;

        let mut replaced = prettified;
        for replacer in &self.regex_replacers {
            replaced = replacer.replace_all(&replaced)?.to_string();
        }

        Ok(replaced)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        if self.css_selector.is_some() {
            return Err(anyhow::anyhow!(
                "css_selector is now an array and named css_selectors"
            ));
        }

        for rp in &self.regex_replacers {
            rp.is_valid()?;
        }

        Ok(())
    }
}
