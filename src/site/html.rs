use scraper::Selector;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Response;
use crate::regex_replacer::RegexReplacer;

use super::url_filename;

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
        let content = if let Some(selector) = &self.css_selector {
            let selector = Selector::parse(selector).unwrap();
            let html = scraper::Html::parse_document(&content);
            let selected = html.select(&selector).map(|o| o.html()).collect::<Vec<_>>();
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
            if let Err(err) = Selector::parse(selector) {
                return Err(anyhow::anyhow!(
                    "css selector ({}) parse error: {:?}",
                    selector,
                    err
                ));
            }
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
