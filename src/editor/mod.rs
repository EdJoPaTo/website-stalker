use anyhow::anyhow;
use serde::Deserialize;
use url::Url;

pub mod css_remove;
pub mod css_selector;
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    CssRemove(#[serde(deserialize_with = "deserialize_selector")] scraper::Selector),
    CssSelect(#[serde(deserialize_with = "deserialize_selector")] scraper::Selector),
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

    fn apply(&self, url: &Url, input: &Content) -> anyhow::Result<Content> {
        match &self {
            Self::CssRemove(s) => Ok(Content {
                extension: Some("html"),
                text: css_remove::apply(s, &input.text),
            }),
            Self::CssSelect(s) => Ok(Content {
                extension: Some("html"),
                text: css_selector::apply(s, &input.text)?,
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
        for (i, e) in editors.iter().enumerate() {
            content = e
                .apply(url, &content)
                .map_err(|err| anyhow!("in editor[{i}] {}: {err}", e.log_name()))?;
        }
        Ok(content)
    }
}

fn deserialize_selector<'de, D>(deserializer: D) -> Result<scraper::Selector, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    scraper::Selector::parse(&s).map_err(serde::de::Error::custom)
}

fn deserialize_selector_opt<'de, D>(deserializer: D) -> Result<Option<scraper::Selector>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(deserialize_selector(deserializer)?))
}
