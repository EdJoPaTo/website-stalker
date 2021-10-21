use serde::{Deserialize, Serialize};
use url::Url;

pub mod css_remove;
pub mod css_selector;
pub mod html_markdown;
pub mod html_pretty;
pub mod html_text;
pub mod html_url;
pub mod json_prettify;
pub mod regex_replacer;
pub mod rss;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    CssRemove(css_remove::CssRemover),
    CssSelect(css_selector::CssSelector),
    HtmlMarkdownify,
    HtmlPrettify,
    HtmlTextify,
    HtmlUrlCanonicalize,
    JsonPrettify,
    RegexReplace(regex_replacer::RegexReplacer),
    Rss(rss::Rss),
}

impl Editor {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        match &self {
            Editor::CssRemove(e) => e.is_valid()?,
            Editor::CssSelect(e) => e.is_valid()?,
            Editor::RegexReplace(e) => e.is_valid()?,
            Editor::Rss(e) => e.is_valid()?,
            Editor::HtmlMarkdownify
            | Editor::HtmlPrettify
            | Editor::HtmlTextify
            | Editor::HtmlUrlCanonicalize
            | Editor::JsonPrettify => {}
        }
        Ok(())
    }

    pub fn apply(&self, url: &Url, input: &str) -> anyhow::Result<String> {
        match &self {
            Editor::CssRemove(e) => e.apply(input),
            Editor::CssSelect(e) => e.apply(input),
            Editor::HtmlMarkdownify => html_markdown::markdownify(input),
            Editor::HtmlPrettify => html_pretty::prettify(input),
            Editor::HtmlTextify => html_text::textify(input),
            Editor::HtmlUrlCanonicalize => html_url::canonicalize(url, input),
            Editor::JsonPrettify => json_prettify::prettify(input),
            Editor::RegexReplace(e) => Ok(e.replace_all(input)?.to_string()),
            Editor::Rss(e) => e.generate(url, input),
        }
    }
}

pub fn apply_many(editors: &[Editor], url: &Url, mut content: String) -> anyhow::Result<String> {
    for e in editors {
        content = e.apply(url, &content)?;
    }
    Ok(content)
}
