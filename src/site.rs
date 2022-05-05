use serde::{Deserialize, Serialize};
use url::Url;

use crate::editor::Editor;
use crate::filename;

#[derive(Debug)]
pub struct Site {
    pub url: Url,
    pub options: Options,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub accept_invalid_certs: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ignore_error: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Editor>,
}

impl Site {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.options.is_valid()
    }

    pub fn get_all_file_basenames(sites: &[Self]) -> Vec<String> {
        sites.iter().map(|o| filename::basename(&o.url)).collect()
    }

    pub fn validate_no_duplicate(sites: &[Self]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut file_basenames = Self::get_all_file_basenames(sites);
        file_basenames.sort_unstable();
        let total = file_basenames.len();
        file_basenames.dedup();
        if file_basenames.len() == total {
            Ok(())
        } else {
            Err("Some sites are duplicates of each other".to_string())
        }
    }
}

impl Options {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        for e in &self.editors {
            e.is_valid()?;
        }
        Ok(())
    }
}

#[test]
#[should_panic = "duplicates"]
fn validate_finds_duplicates() {
    let sites = vec![
        Site {
            url: Url::parse("https://edjopato.de/post/").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                editors: vec![],
            },
        },
    ];
    Site::validate_no_duplicate(&sites).unwrap();
}
