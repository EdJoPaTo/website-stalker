use serde::{Deserialize, Serialize};
use url::Url;

use crate::serde_helper::string_or_struct;

pub mod css_selector;
pub mod html_prettify;
pub mod html_text;
pub mod regex_replacer;
pub mod rss;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    #[serde(deserialize_with = "string_or_struct")]
    CssSelector(css_selector::CssSelector),
    HtmlPrettify,
    HtmlText,
    RegexReplacer(regex_replacer::RegexReplacer),
    Rss(rss::Rss),
}

impl Editor {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        match &self {
            Editor::CssSelector(e) => e.is_valid()?,
            Editor::RegexReplacer(e) => e.is_valid()?,
            Editor::Rss(e) => e.is_valid()?,
            Editor::HtmlPrettify | Editor::HtmlText => {}
        }
        Ok(())
    }

    pub fn apply(&self, url: &Url, input: &str) -> anyhow::Result<String> {
        match &self {
            Editor::CssSelector(e) => e.apply(input),
            Editor::HtmlPrettify => html_prettify::prettify(input),
            Editor::HtmlText => html_text::extract(input),
            Editor::RegexReplacer(e) => Ok(e.replace_all(input)?.to_string()),
            Editor::Rss(e) => e.generate(url, input),
        }
    }
}
