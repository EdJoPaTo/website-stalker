use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

mod html;
mod utf8;

#[derive(Debug, Deserialize, Serialize)]
pub enum Site {
    Html(html::Html),
    Utf8(utf8::Utf8),
}

pub trait Huntable {
    fn get_filename(&self) -> String;
    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String>;
}

impl Huntable for Site {
    fn get_filename(&self) -> String {
        match self {
            Site::Html(o) => o.get_filename(),
            Self::Utf8(o) => o.get_filename(),
        }
    }

    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String> {
        match self {
            Site::Html(o) => o.hunt(http_agent),
            Site::Utf8(o) => o.hunt(http_agent),
        }
    }
}

impl Site {
    pub fn examples() -> Vec<Site> {
        vec![
            Site::Html(html::Html {
                url: Url::parse("https://edjopato.de/post/").unwrap(),
            }),
            Site::Utf8(utf8::Utf8 {
                url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            }),
        ]
    }
}

fn format_url_as_filename(url: &url::Url, extension: &str) -> String {
    let re = Regex::new("[^a-zA-Z\\d]+").unwrap();
    let only_ascii = re.replace_all(url.as_str(), "-");
    let trimmed = only_ascii.trim_matches('-');
    format!("{}.{}", trimmed, extension)
}
