use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Response;
use crate::regex_replacer::RegexReplacer;

use super::url_filename;

mod css_selection;
mod prettify;

#[derive(Debug, Deserialize, Serialize)]
pub struct Html {
    pub url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,

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
        let content = if let Some(selector_str) = &self.css_selector {
            let selected = css_selection::select(&content, selector_str)?;
            if selected.is_empty() {
                return Err(anyhow::anyhow!(
                    "css_selector ({}) selected nothing",
                    selector_str
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
        if let Some(selector) = &self.css_selector {
            css_selection::selector_is_valid(selector)?;
        }

        for rp in &self.regex_replacers {
            rp.is_valid()?;
        }

        Ok(())
    }
}

#[test]
fn css_selector_valid() {
    let example = Html {
        url: Url::parse("https://edjopato.de/").unwrap(),
        css_selector: Some("body".to_string()),
        regex_replacers: vec![],
    };
    let result = example.is_valid();
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn css_selector_invalid() {
    let example = Html {
        url: Url::parse("https://edjopato.de/").unwrap(),
        css_selector: Some(".".to_string()),
        regex_replacers: vec![],
    };
    let result = example.is_valid();
    println!("{:?}", result);
    assert!(result.is_err());
}
