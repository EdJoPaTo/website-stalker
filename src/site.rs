use serde::{Deserialize, Serialize};
use url::Url;

use crate::editor::Editor;

mod filename;

#[derive(Debug)]
pub struct Site {
    pub url: Url,
    pub options: Options,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    pub extension: String,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub accept_invalid_certs: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Editor>,
}

impl Site {
    pub fn get_filename(&self) -> String {
        filename::format(&self.url, &self.options.extension)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.options.is_valid()
    }

    pub fn get_all_filenames(sites: &[Site]) -> Vec<String> {
        sites.iter().map(Site::get_filename).collect::<Vec<_>>()
    }

    pub fn validate_no_duplicate(sites: &[Site]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut filenames = Self::get_all_filenames(sites);
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

impl Options {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        if self.extension.is_empty() || self.extension.len() > 12 || !self.extension.is_ascii() {
            return Err(anyhow::anyhow!(
                "extension has to be a short ascii filename extension"
            ));
        }

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
                extension: "html".to_string(),
                accept_invalid_certs: false,
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            options: Options {
                extension: "txt".to_string(),
                accept_invalid_certs: false,
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            options: Options {
                extension: "html".to_string(),
                accept_invalid_certs: false,
                editors: vec![],
            },
        },
    ];
    Site::validate_no_duplicate(&sites).unwrap();
}
