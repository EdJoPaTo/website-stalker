use anyhow::anyhow;
use serde::Deserialize;
use url::Url;

use crate::final_message::validate_template;
use crate::http::validate_from;
use crate::site::{Options, Site};

pub const EXAMPLE_CONF: &str = include_str!("../sites/website-stalker.yaml");

#[derive(Debug, Deserialize)]
pub struct Config {
    // Read as empty string when not defined as it could be overridden from the env
    #[serde(default)]
    pub from: String,

    #[serde(
        default,
        deserialize_with = "deserialize_mustache_template",
        skip_serializing_if = "Option::is_none"
    )]
    pub notification_template: Option<mustache::Template>,

    pub sites: Vec<SiteEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UrlVariants {
    Single(Url),
    Many(Vec<Url>),
}

impl UrlVariants {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Many(many) => many.is_empty(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SiteEntry {
    pub url: UrlVariants,
    #[serde(flatten)]
    pub options: Options,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let filecontent = std::fs::read_to_string("website-stalker.yaml")?;
        let mut config = serde_yaml::from_str::<Self>(&filecontent)?;

        if let Ok(from) = std::env::var("WEBSITE_STALKER_FROM") {
            config.from = from;
        }

        config.validate()?;
        Ok(config)
    }

    pub fn get_sites(&self) -> Vec<Site> {
        let mut result = Vec::new();
        for entry in &self.sites {
            match &entry.url {
                UrlVariants::Single(url) => result.push(Site {
                    url: url.clone(),
                    options: entry.options.clone(),
                }),
                UrlVariants::Many(many) => {
                    for url in many {
                        result.push(Site {
                            url: url.clone(),
                            options: entry.options.clone(),
                        });
                    }
                }
            }
        }
        result
    }

    fn validate(&self) -> anyhow::Result<()> {
        validate_from(&self.from)
            .map_err(|err| anyhow!("from ({}) is invalid: {err}", self.from))?;
        self.validate_sites()?;
        Ok(())
    }

    fn validate_sites(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.sites.is_empty(), "site list is empty");
        for entry in &self.sites {
            anyhow::ensure!(!entry.url.is_empty(), "site entry has no urls");
        }

        let sites = self.get_sites();
        Site::validate_no_duplicate(&sites)?;
        Ok(())
    }
}

fn deserialize_mustache_template<'de, D>(
    deserializer: D,
) -> Result<Option<mustache::Template>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    let template = mustache::compile_str(&str).map_err(serde::de::Error::custom)?;
    validate_template(&template).map_err(serde::de::Error::custom)?;
    Ok(Some(template))
}

#[test]
fn example_sites_are_valid() {
    let config = serde_yaml::from_str::<Config>(EXAMPLE_CONF).unwrap();
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site list is empty"]
fn validate_fails_on_empty_sites_list() {
    let config = Config {
        from: "dummy".to_owned(),
        notification_template: None,
        sites: vec![],
    };
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site entry has no urls"]
fn validate_fails_on_sites_list_with_empty_many() {
    let config = Config {
        from: "dummy".to_owned(),
        notification_template: None,
        sites: vec![SiteEntry {
            url: UrlVariants::Many(vec![]),
            options: Options {
                accept_invalid_certs: false,
                ignore_error: false,
                headers: reqwest::header::HeaderMap::new(),
                editors: vec![],
                filename: None,
            },
        }],
    };
    config.validate_sites().unwrap();
}
