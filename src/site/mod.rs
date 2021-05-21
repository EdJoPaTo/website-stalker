use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

pub use self::general::Huntable;

mod general;
mod html;
mod utf8;

#[derive(Debug, Deserialize, Serialize)]
pub enum Site {
    Html(html::Html),
    Utf8(utf8::Utf8),
}

impl Huntable for Site {
    fn hunt(&self, http_agent: &Http) -> anyhow::Result<general::HuntOutput> {
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
