use std::path::{Path, PathBuf};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use url::Url;

use crate::editor::Editor;
use crate::filename;

#[derive(Debug)]
pub struct Site {
    pub url: Url,
    pub options: Options,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Options {
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub accept_invalid_certs: bool,

    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub http1_only: bool,

    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub ignore_error: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<PathBuf>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_headermap"
    )]
    pub headers: HeaderMap,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Editor>,
}

impl Site {
    pub fn to_file_path(&self) -> PathBuf {
        self.options.filename.clone().unwrap_or_else(|| {
            let folder = filename::domainfolder(&self.url);
            let [first, rest @ ..] = folder.as_slice() else {
                unreachable!(
                    "domain has to have at least one segment {folder:?} {:?}",
                    self.url
                );
            };
            let mut path = Path::new(first).to_path_buf();
            for folder in rest {
                path = path.join(folder);
            }
            path.join(filename::filename(&self.url))
        })
    }

    fn unique_idenfier(&self) -> String {
        self.to_file_path()
            .to_str()
            .expect("the path should be unicode already")
            .to_owned()
    }

    pub fn get_all_file_paths(sites: &[Self]) -> Vec<PathBuf> {
        sites.iter().map(Self::to_file_path).collect()
    }

    pub fn validate_no_duplicate(sites: &[Self]) -> anyhow::Result<()> {
        // TODO: return url or something of specific duplicates
        let mut uniq = sites.iter().map(Self::unique_idenfier).collect::<Vec<_>>();
        uniq.sort_unstable();
        let total = uniq.len();
        uniq.dedup();
        anyhow::ensure!(
            uniq.len() == total,
            "Some sites are duplicates of each other or result in the same filename."
        );
        Ok(())
    }
}

fn deserialize_headermap<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let headers = Vec::<String>::deserialize(deserializer)?;
    let mut result = HeaderMap::new();
    for entry in headers {
        let (key, value) = entry.split_once(": ").ok_or_else(|| {
            serde::de::Error::custom("does not contain ': ' to separate header key/value")
        })?;
        let key = key
            .parse::<HeaderName>()
            .map_err(serde::de::Error::custom)?;
        let value = value
            .parse::<HeaderValue>()
            .map_err(serde::de::Error::custom)?;
        result.append(key, value);
    }
    Ok(result)
}

#[test]
#[should_panic = "duplicates"]
fn validate_finds_duplicates() {
    let sites = vec![
        Site {
            url: Url::parse("https://edjopato.de/post/").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                http1_only: false,
                ignore_error: false,
                filename: None,
                headers: HeaderMap::new(),
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                http1_only: false,
                ignore_error: false,
                filename: None,
                headers: HeaderMap::new(),
                editors: vec![],
            },
        },
        Site {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            options: Options {
                accept_invalid_certs: false,
                http1_only: false,
                ignore_error: false,
                filename: None,
                headers: HeaderMap::new(),
                editors: vec![],
            },
        },
    ];
    Site::validate_no_duplicate(&sites).unwrap();
}
