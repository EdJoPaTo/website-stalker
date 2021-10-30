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

pub struct Content {
    pub extension: Option<&'static str>,
    pub text: String,
}

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

    pub fn apply(&self, url: &Url, input: &Content) -> anyhow::Result<Content> {
        match &self {
            Editor::CssRemove(e) => Ok(Content {
                extension: Some("html"),
                text: e.apply(&input.text)?,
            }),
            Editor::CssSelect(e) => Ok(Content {
                extension: Some("html"),
                text: e.apply(&input.text)?,
            }),
            Editor::HtmlMarkdownify => Ok(Content {
                extension: Some("md"),
                text: html_markdown::markdownify(&input.text)?,
            }),
            Editor::HtmlPrettify => Ok(Content {
                extension: Some("html"),
                text: html_pretty::prettify(&input.text)?,
            }),
            Editor::HtmlTextify => Ok(Content {
                extension: Some("txt"),
                text: html_text::textify(&input.text)?,
            }),
            Editor::HtmlUrlCanonicalize => Ok(Content {
                extension: Some("html"),
                text: html_url::canonicalize(url, &input.text)?,
            }),
            Editor::JsonPrettify => Ok(Content {
                extension: Some("json"),
                text: json_prettify::prettify(&input.text)?,
            }),
            Editor::RegexReplace(e) => Ok(Content {
                extension: input.extension,
                text: e.replace_all(&input.text)?.to_string(),
            }),
            Editor::Rss(e) => Ok(Content {
                extension: Some("xml"),
                text: e.generate(url, &input.text)?,
            }),
        }
    }
}

pub fn apply_many(editors: &[Editor], url: &Url, mut content: Content) -> anyhow::Result<Content> {
    for e in editors {
        content = e.apply(url, &content)?;
    }
    Ok(content)
}
