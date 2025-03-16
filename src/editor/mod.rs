use std::path::PathBuf;

use anyhow::Context as _;
use schemars::JsonSchema;
use serde::Deserialize;
use url::Url;

pub mod css_flatten;
pub mod css_remove;
pub mod css_selector;
pub mod css_sort;
pub mod debug_files;
pub mod html_markdown;
pub mod html_pretty;
pub mod html_sanitize;
pub mod html_text;
pub mod html_url;
pub mod json_prettify;
pub mod regex_replacer;
pub mod rss;

pub struct Content {
    pub extension: Option<&'static str>,
    pub text: String,
}

/// # Editor
/// Editors are manipulating the content of a webpage to simplify comparing them later on.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum Editor {
    CssFlatten(#[schemars(with = "String")] scraper::Selector),
    CssRemove(#[schemars(with = "String")] scraper::Selector),
    CssSelect(#[schemars(with = "String")] scraper::Selector),
    CssSort(css_sort::CssSort),
    DebugFiles(PathBuf),
    HtmlMarkdownify,
    HtmlPrettify,
    HtmlSanitize,
    HtmlTextify,
    HtmlUrlCanonicalize,
    JsonPrettify,
    RegexReplace(regex_replacer::RegexReplacer),
    Rss(rss::Rss),
}

impl Editor {
    pub const fn log_name(&self) -> &'static str {
        match self {
            Self::CssFlatten(_) => "css_flatten",
            Self::CssRemove(_) => "css_remove",
            Self::CssSelect(_) => "css_select",
            Self::CssSort(_) => "css_sort",
            Self::DebugFiles(_) => "debug_files",
            Self::HtmlMarkdownify => "html_markdownify",
            Self::HtmlPrettify => "html_prettify",
            Self::HtmlSanitize => "html_sanitize",
            Self::HtmlTextify => "html_textify",
            Self::HtmlUrlCanonicalize => "html_url_canonicalize",
            Self::JsonPrettify => "json_prettify",
            Self::RegexReplace(_) => "regex_replace",
            Self::Rss(_) => "rss",
        }
    }

    fn apply(&self, url: &Url, input: Content) -> anyhow::Result<Content> {
        match &self {
            Self::CssFlatten(selector) => Ok(Content {
                extension: Some("html"),
                text: css_flatten::apply(selector, &input.text),
            }),
            Self::CssRemove(selector) => Ok(Content {
                extension: Some("html"),
                text: css_remove::apply(selector, &input.text),
            }),
            Self::CssSelect(selector) => Ok(Content {
                extension: Some("html"),
                text: css_selector::apply(selector, &input.text)?,
            }),
            Self::CssSort(sort) => Ok(Content {
                extension: Some("html"),
                text: sort.apply(url, &input.text),
            }),
            Self::DebugFiles(path) => debug_files::debug_files(path, input),
            Self::HtmlMarkdownify => Ok(Content {
                extension: Some("md"),
                text: html_markdown::markdownify(&input.text),
            }),
            Self::HtmlPrettify => Ok(Content {
                extension: Some("html"),
                text: html_pretty::prettify(&input.text)?,
            }),
            Self::HtmlSanitize => Ok(Content {
                extension: Some("html"),
                text: html_sanitize::sanitize(&input.text),
            }),
            Self::HtmlTextify => Ok(Content {
                extension: Some("txt"),
                text: html_text::textify(&input.text)?,
            }),
            Self::HtmlUrlCanonicalize => Ok(Content {
                extension: Some("html"),
                text: html_url::canonicalize(url, &input.text)?,
            }),
            Self::JsonPrettify => Ok(Content {
                extension: Some("json"),
                text: json_prettify::prettify(&input.text)?,
            }),
            Self::RegexReplace(rr) => Ok(Content {
                extension: input.extension,
                text: rr.replace_all(&input.text).to_string(),
            }),
            Self::Rss(rss) => Ok(Content {
                extension: Some("xml"),
                text: rss.generate(url, &input.text)?,
            }),
        }
    }

    pub fn apply_many(
        editors: &[Self],
        url: &Url,
        mut content: Content,
    ) -> anyhow::Result<Content> {
        for (i, editor) in editors.iter().enumerate() {
            content = editor
                .apply(url, content)
                .with_context(|| format!("in editor[{i}] {}", editor.log_name()))?;
        }
        Ok(content)
    }
}
