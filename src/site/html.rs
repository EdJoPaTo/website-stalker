use scraper::Selector;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

use super::url_filename;
use super::Huntable;

mod prettify;

#[derive(Debug, Deserialize, Serialize)]
pub struct Html {
    pub url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,
}

impl Huntable for Html {
    fn get_filename(&self) -> String {
        url_filename::format(&self.url, "html")
    }

    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String> {
        let content = http_agent.get(self.url.as_str())?;

        #[allow(clippy::option_if_let_else)]
        let content = if let Some(selector) = &self.css_selector {
            let selector = Selector::parse(selector).unwrap();
            let html = scraper::Html::parse_document(&content);
            let selected = html.select(&selector).map(|o| o.html()).collect::<Vec<_>>();
            selected.join("\n")
        } else {
            content
        };

        let result = prettify::prettify(&content)?;
        Ok(result)
    }

    fn is_valid(&self) -> anyhow::Result<()> {
        if let Some(selector) = &self.css_selector {
            if let Err(err) = Selector::parse(selector) {
                return Err(anyhow::anyhow!(
                    "css selector ({}) parse error: {:?}",
                    selector,
                    err
                ));
            }
        }

        Ok(())
    }
}

#[test]
fn css_selector_valid() {
    let example = Html {
        url: Url::parse("https://edjopato.de/").unwrap(),
        css_selector: Some("body".to_string()),
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
    };
    let result = example.is_valid();
    println!("{:?}", result);
    assert!(result.is_err());
}
