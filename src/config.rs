use std::fs;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::http::validate_from;
use crate::site::Site;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub from: String,

    pub sites: Vec<Site>,
}

impl Config {
    pub fn example() -> Self {
        Self {
            from: "my-email-address".to_string(),
            sites: Site::examples(),
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

    fn validate(&self) -> anyhow::Result<()> {
        validate_from(&self.from)
            .map_err(|err| anyhow!("from ({}) is invalid: {}", self.from, err))?;
        self.validate_min_one_site()?;
        Site::validate_no_duplicate(&self.sites).map_err(|err| anyhow!("{}", err))?;
        self.validate_each_site()?;
        Ok(())
    }

    fn validate_min_one_site(&self) -> anyhow::Result<()> {
        if self.sites.is_empty() {
            return Err(anyhow!("site list is empty"));
        }
        Ok(())
    }

    fn validate_each_site(&self) -> anyhow::Result<()> {
        for site in &self.sites {
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
    config.validate_each_site().unwrap();
}

#[test]
fn validate_fails_on_empty_sites_list() {
    let config = Config {
        from: "dummy".to_string(),
        sites: vec![],
    };
    let result = config.validate_min_one_site();
    assert!(result.is_err());
}
