use serde::{Deserialize, Serialize};
use url::Url;

use crate::serde_helper::string_or_struct;

pub mod css_selector;
pub mod html_markdown;
pub mod html_pretty;
pub mod html_text;
pub mod regex_replacer;
pub mod rss;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    #[serde(deserialize_with = "string_or_struct")]
    CssSelect(css_selector::CssSelector),
    HtmlMarkdownify,
    HtmlPrettify,
    HtmlTextify,
    RegexReplace(regex_replacer::RegexReplacer),
    Rss(rss::Rss),
}

impl Editor {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        match &self {
            Editor::CssSelect(e) => e.is_valid()?,
            Editor::RegexReplace(e) => e.is_valid()?,
            Editor::Rss(e) => e.is_valid()?,
            Editor::HtmlMarkdownify | Editor::HtmlPrettify | Editor::HtmlTextify => {}
        }
        Ok(())
    }

    pub fn apply(&self, url: &Url, input: &str) -> anyhow::Result<String> {
        match &self {
            Editor::CssSelect(e) => e.apply(input),
            Editor::HtmlMarkdownify => html_markdown::markdownify(input),
            Editor::HtmlPrettify => html_pretty::prettify(input),
            Editor::HtmlTextify => html_text::textify(input),
            Editor::RegexReplace(e) => Ok(e.replace_all(input)?.to_string()),
            Editor::Rss(e) => e.generate(url, input),
        }
    }
}