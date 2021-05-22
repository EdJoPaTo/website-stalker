use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Http;

mod html;
mod url_filename;
mod utf8;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Site {
    Html(html::Html),
    Utf8(utf8::Utf8),
}

pub trait Huntable {
    fn get_filename(&self) -> String;
    fn hunt(&self, http_agent: &Http) -> anyhow::Result<String>;
    fn is_valid(&self) -> anyhow::Result<()>;
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

    fn is_valid(&self) -> anyhow::Result<()> {
        match self {
            Site::Html(o) => o.is_valid(),
            Site::Utf8(o) => o.is_valid(),
        }
    }
}

impl Site {
    pub fn examples() -> Vec<Site> {
        vec![
            Site::Html(html::Html {
                url: Url::parse("https://edjopato.de/post/").unwrap(),
                css_selector: Some("section".to_string()),
            }),
            Site::Utf8(utf8::Utf8 {
                url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            }),
        ]
    }

    pub fn validate_no_duplicate(sites: &[Site]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut filenames = sites.iter().map(|o| o.get_filename()).collect::<Vec<_>>();
        filenames.sort_unstable();
        let filename_amount = filenames.len();
        filenames.dedup();
        if filenames.len() == filename_amount {
            Ok(())
        } else {
            Err("Some sites are duplicates of each other".to_string())
        }
    }
}

#[test]
fn validate_finds_duplicates() {
    let sites = vec![
        Site::Html(html::Html {
            url: Url::parse("https://edjopato.de/post/").unwrap(),
            css_selector: None,
        }),
        Site::Utf8(utf8::Utf8 {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
        }),
        Site::Html(html::Html {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            css_selector: None,
        }),
    ];

    let result = Site::validate_no_duplicate(&sites);
    println!("{:?}", result);
    assert!(result.is_err());
}
