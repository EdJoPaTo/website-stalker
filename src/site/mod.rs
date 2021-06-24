use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::Response;

mod html;
mod url_filename;
mod utf8;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Site {
    Html(html::Html),
    Utf8(utf8::Utf8),
}

impl Site {
    pub fn get_filename(&self) -> String {
        match self {
            Site::Html(o) => o.get_filename(),
            Self::Utf8(o) => o.get_filename(),
        }
    }

    pub async fn stalk(&self, response: Response) -> anyhow::Result<String> {
        match self {
            Site::Html(o) => o.stalk(response).await,
            Site::Utf8(o) => o.stalk(response).await,
        }
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        match self {
            Site::Html(o) => o.is_valid(),
            Site::Utf8(o) => o.is_valid(),
        }
    }

    pub fn examples() -> Vec<Site> {
        vec![
            Site::Html(html::Html {
                url: Url::parse("https://edjopato.de/post/").unwrap(),
                css_selector: Some(html::CssSelector::parse("section").unwrap()),
                regex_replacers: vec![],
            }),
            Site::Utf8(utf8::Utf8 {
                url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
                regex_replacers: vec![],
            }),
        ]
    }

    pub fn validate_no_duplicate(sites: &[Site]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut filenames = sites.iter().map(Site::get_filename).collect::<Vec<_>>();
        filenames.sort_unstable();
        let filename_amount = filenames.len();
        filenames.dedup();
        if filenames.len() == filename_amount {
            Ok(())
        } else {
            Err("Some sites are duplicates of each other".to_string())
        }
    }

    pub fn get_url(&self) -> &Url {
        match self {
            Site::Html(o) => &o.url,
            Site::Utf8(o) => &o.url,
        }
    }
}

#[test]
fn validate_finds_duplicates() {
    let sites = vec![
        Site::Html(html::Html {
            url: Url::parse("https://edjopato.de/post/").unwrap(),
            css_selector: None,
            regex_replacers: vec![],
        }),
        Site::Utf8(utf8::Utf8 {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            regex_replacers: vec![],
        }),
        Site::Html(html::Html {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            css_selector: None,
            regex_replacers: vec![],
        }),
    ];

    let result = Site::validate_no_duplicate(&sites);
    println!("{:?}", result);
    assert!(result.is_err());
}
