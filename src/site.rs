use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::editor::Editor;
use crate::filename;

#[derive(Debug)]
pub struct Site {
    pub url: Url,
    pub options: Options,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Options {
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub accept_invalid_certs: bool,

    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub ignore_error: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Editor>,
}

impl Site {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.options.is_valid()
    }

    pub fn to_file_path(&self) -> PathBuf {
        self.options.filename.clone().unwrap_or_else(|| {
            Path::new(&filename::domainfolder(&self.url)).join(filename::filename(&self.url))
        })
    }

    fn unique_idenfier(&self) -> String {
        self.to_file_path()
            .to_str()
            .expect("the path is unicode already")
            .to_string()
    }

    pub fn get_all_file_paths(sites: &[Self]) -> Vec<PathBuf> {
        sites.iter().map(Self::to_file_path).collect()
    }

    pub fn validate_no_duplicate(sites: &[Self]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut uniq = sites.iter().map(Self::unique_idenfier).collect::<Vec<_>>();
        uniq.sort_unstable();
        let total = uniq.len();
        uniq.dedup();
        if uniq.len() == total {
            Ok(())
        } else {
            Err(
                "Some sites are duplicates of each other or result in the same filename."
                    .to_string(),
            )
        }
    }
}

impl Options {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        for entry in &self.headers {
            let (k, v) = entry.split_once(": ").ok_or_else(|| {
                anyhow::anyhow!("does not contain ': ' to separate header key/value: {entry}")
            })?;
            k.parse::<reqwest::header::HeaderName>()?;
            v.parse::<reqwest::header::HeaderValue>()?;
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
                accept_invalid_certs: false,
                ignore_error: false,
                headers: Vec::new(),
                editors: vec![],
                filename: None,
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                headers: Vec::new(),
                editors: vec![],
                filename: None,
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                headers: Vec::new(),
                editors: vec![],
                filename: None,
            },
        },
    ];
    Site::validate_no_duplicate(&sites).unwrap();
}
