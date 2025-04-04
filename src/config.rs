use anyhow::Context as _;
use schemars::JsonSchema;
use serde::Deserialize;
use url::Url;

use crate::http::validate_from;
use crate::logger;
use crate::site::{Options, Site};

/// # Website Stalker configuration file
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Read as empty string when not defined as it could be overridden from the env
    #[serde(default)]
    #[schemars(email)]
    pub from: String,

    pub sites: Vec<SiteEntry>,
}

/// Single or multiple URLs
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum UrlVariants {
    Single(Url),
    Many(#[schemars(length(min = 1))] Vec<Url>),
}

impl UrlVariants {
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Many(many) => many.is_empty(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SiteEntry {
    pub url: UrlVariants,
    #[serde(flatten)]
    pub options: Options,
}

impl Config {
    pub const EXAMPLE: &str = include_str!("../sites/website-stalker.yaml");

    pub fn load(cli_from: Option<String>) -> anyhow::Result<Self> {
        let filecontent = std::fs::read_to_string("website-stalker.yaml")?;
        let mut config = serde_yaml::from_str::<Self>(&filecontent)?;

        if let Some(from) = cli_from {
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
        const OLD_PLING_ENV_VARS: [&str; 20] = [
            "EMAIL_FROM",
            "EMAIL_PASSWORD",
            "EMAIL_PORT",
            "EMAIL_SERVER",
            "EMAIL_SUBJECT",
            "EMAIL_TO",
            "EMAIL_USERNAME",
            "MATRIX_ACCESS_TOKEN",
            "MATRIX_HOMESERVER",
            "MATRIX_ROOM_ID",
            "PLING_COMMAND_ARGS",
            "PLING_COMMAND_PROGRAM",
            "PLING_DESKTOP_ENABLED",
            "PLING_DESKTOP_SUMMARY",
            "SLACK_HOOK",
            "TELEGRAM_BOT_TOKEN",
            "TELEGRAM_DISABLE_NOTIFICATION",
            "TELEGRAM_DISABLE_WEB_PAGE_PREVIEW",
            "TELEGRAM_TARGET_CHAT",
            "WEBHOOK_URL",
        ];

        validate_from(&self.from).with_context(|| format!("from ({}) is invalid", self.from))?;
        self.validate_sites()?;

        for (key, _value) in std::env::vars_os().filter(|(key, _value)| {
            key.to_str()
                .is_some_and(|key| OLD_PLING_ENV_VARS.contains(&key))
        }) {
            logger::warn(&format!(
                "Environment variable {} was part of the old notification setup. Check website-stalker run --help for the new notification settings.",
                key.display())
            );
        }

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

#[test]
fn example_sites_are_valid() {
    let config = serde_yaml::from_str::<Config>(Config::EXAMPLE).unwrap();
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site list is empty"]
fn validate_fails_on_empty_sites_list() {
    let config = Config {
        from: "dummy".to_owned(),
        sites: vec![],
    };
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site entry has no urls"]
fn validate_fails_on_sites_list_with_empty_many() {
    let config = Config {
        from: "dummy".to_owned(),
        sites: vec![SiteEntry {
            url: UrlVariants::Many(vec![]),
            options: Options {
                accept_invalid_certs: false,
                http1_only: false,
                ignore_error: false,
                filename: None,
                headers: reqwest::header::HeaderMap::new(),
                editors: vec![],
            },
        }],
    };
    config.validate_sites().unwrap();
}
