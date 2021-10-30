use std::fs;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::editor::regex_replacer::RegexReplacer;
use crate::editor::Editor;
use crate::http::validate_from;
use crate::site::{Options, Site};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub from: String,
    sites: Vec<SiteEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum UrlVariants {
    Single(Url),
    Many(Vec<Url>),
}

impl From<Url> for UrlVariants {
    fn from(url: Url) -> Self {
        Self::Single(url)
    }
}

impl UrlVariants {
    pub fn is_empty(&self) -> bool {
        match self {
            UrlVariants::Single(_) => false,
            UrlVariants::Many(many) => many.is_empty(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SiteEntry {
    url: UrlVariants,
    #[serde(flatten)]
    options: Options,
}

impl Config {
    pub fn example() -> Self {
        Self {
            from: "my-email-address".to_string(),
            sites: vec![
                SiteEntry {
                    url: Url::parse("https://edjopato.de/post/").unwrap().into(),
                    options: Options {
                        accept_invalid_certs: false,
                        editors: vec![
                            Editor::CssSelect("article".parse().unwrap()),
                            Editor::CssRemove("a".parse().unwrap()),
                            Editor::HtmlPrettify,
                            Editor::RegexReplace(RegexReplacer {
                                pattern: "(Lesezeit): \\d+ \\w+".to_string(),
                                replace: "$1".to_string(),
                            }),
                        ],
                    },
                },
                SiteEntry {
                    url: Url::parse("https://edjopato.de/robots.txt").unwrap().into(),
                    options: Options {
                        accept_invalid_certs: false,
                        editors: vec![],
                    },
                },
            ],
        }
    }

    pub fn example_yaml_string() -> String {
        serde_yaml::to_string(&Self::example()).unwrap()
    }

    pub fn load_yaml_file() -> anyhow::Result<Self> {
        let config: Self = serde_yaml::from_str(&fs::read_to_string("website-stalker.yaml")?)?;
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
            .map_err(|err| anyhow!("from ({}) is invalid: {}", self.from, err))?;
        self.validate_sites()?;
        Ok(())
    }

    fn validate_sites(&self) -> anyhow::Result<()> {
        if self.sites.is_empty() {
            return Err(anyhow!("site list is empty"));
        }
        for entry in &self.sites {
            if entry.url.is_empty() {
                return Err(anyhow!("site entry has no urls"));
            }
        }

        let sites = self.get_sites();
        Site::validate_no_duplicate(&sites).map_err(|err| anyhow!("{}", err))?;
        for site in sites {
            if let Err(err) = site.is_valid() {
                return Err(anyhow!("site entry is invalid: {}\n{:?}", err, site));
            }
        }
        Ok(())
    }
}

#[test]
fn can_parse_example() {
    Config::example_yaml_string();
}

#[test]
fn example_sites_are_valid() {
    let config = Config::example();
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site list is empty"]
fn validate_fails_on_empty_sites_list() {
    let config = Config {
        from: "dummy".to_string(),
        sites: vec![],
    };
    config.validate_sites().unwrap();
}

#[test]
#[should_panic = "site entry has no urls"]
fn validate_fails_on_sites_list_with_empty_many() {
    let config = Config {
        from: "dummy".to_string(),
        sites: vec![SiteEntry {
            url: UrlVariants::Many(vec![]),
            options: Options {
                accept_invalid_certs: false,
                editors: vec![],
            },
        }],
    };
    config.validate_sites().unwrap();
}
