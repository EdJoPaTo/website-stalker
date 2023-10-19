use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use url::Url;

pub mod css_remove;
pub mod css_selector;
pub mod css_sort;
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    CssRemove(css_remove::CssRemover),
    CssSelect(css_selector::CssSelector),
    CssSort(css_sort::CssSort),
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
        // TODO: can serde do this?
        match self {
            Self::CssRemove(_) => "css_remove",
            Self::CssSelect(_) => "css_select",
            Self::CssSort(_) => "css_sort",
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

    fn is_valid(&self) -> anyhow::Result<()> {
        match &self {
            Self::CssRemove(e) => e.is_valid()?,
            Self::CssSelect(e) => e.is_valid()?,
            Self::CssSort(e) => e.is_valid()?,
            Self::RegexReplace(e) => e.is_valid()?,
            Self::Rss(e) => e.is_valid()?,
            Self::HtmlMarkdownify
            | Self::HtmlPrettify
            | Self::HtmlSanitize
            | Self::HtmlTextify
            | Self::HtmlUrlCanonicalize
            | Self::JsonPrettify => {}
        }
        Ok(())
    }

    fn apply(&self, url: &Url, input: &Content) -> anyhow::Result<Content> {
        match &self {
            Self::CssRemove(e) => Ok(Content {
                extension: Some("html"),
                text: e.apply(&input.text)?,
            }),
            Self::CssSelect(e) => Ok(Content {
                extension: Some("html"),
                text: e.apply(&input.text)?,
            }),
            Self::CssSort(e) => Ok(Content {
                extension: Some("html"),
                text: e.apply(url, &input.text)?,
            }),
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
            Self::RegexReplace(e) => Ok(Content {
                extension: input.extension,
                text: e.replace_all(&input.text)?.to_string(),
            }),
            Self::Rss(e) => Ok(Content {
                extension: Some("xml"),
                text: e.generate(url, &input.text)?,
            }),
        }
    }

    pub fn many_valid(editors: &[Self]) -> anyhow::Result<()> {
        for (i, e) in editors.iter().enumerate() {
            e.is_valid()
                .map_err(|err| anyhow!("in editor[{i}] {}: {err}", e.log_name()))?;
        }
        Ok(())
    }

    pub fn apply_many(
        editors: &[Self],
        url: &Url,
        mut content: Content,
    ) -> anyhow::Result<Content> {
        for (i, e) in editors.iter().enumerate() {
            content = e
                .apply(url, &content)
                .map_err(|err| anyhow!("in editor[{i}] {}: {err}", e.log_name()))?;
        }
        Ok(content)
    }
}
