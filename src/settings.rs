use config::ConfigError;
use serde::{Deserialize, Serialize};

use crate::http::validate_from;
use crate::site::Site;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub from: String,

    pub sites: Vec<Site>,
}

impl Settings {
    pub fn example() -> Self {
        Self {
            from: "my-email-address".to_string(),
            sites: Site::examples(),
        }
    }

    pub fn load() -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();
        settings
            // Add in `./website-stalker.toml`, `./website-stalker.yaml`, ...
            .merge(config::File::with_name("website-stalker").required(false))?
            // Add in settings from the environment (with a prefix of WEBSITE_STALKER)
            // Eg.. `WEBSITE_STALKER_DEBUG=1 network-stalker` would set the `debug` key
            .merge(config::Environment::with_prefix("WEBSITE_STALKER"))?;

        let settings: Self = settings.try_into()?;
        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_from(&self.from).map_err(|err| {
            ConfigError::Message(format!("from ({}) is invalid: {}", self.from, err))
        })?;
        self.validate_min_one_site()?;
        Site::validate_no_duplicate(&self.sites).map_err(ConfigError::Message)?;
        self.validate_each_site()?;
        Ok(())
    }

    fn validate_min_one_site(&self) -> Result<(), ConfigError> {
        if self.sites.is_empty() {
            return Err(ConfigError::Message("site list is empty".to_string()));
        }
        Ok(())
    }

    fn validate_each_site(&self) -> Result<(), ConfigError> {
        for site in &self.sites {
            if let Err(err) = site.is_valid() {
                return Err(ConfigError::Message(format!(
                    "site entry is invalid: {}\n{:?}",
                    err, site,
                )));
            }
        }
        Ok(())
    }
}

#[test]
fn can_parse_example_config() {
    let settings = Settings::example();
    let content = serde_yaml::to_string(&settings);
    println!("{:?}", content);
    assert!(content.is_ok(), "failed to parse default settings to yaml");
}

#[test]
fn validate_fails_on_empty_sites_list() {
    let settings = Settings {
        from: "dummy".to_string(),
        sites: vec![],
    };
    let result = settings.validate_min_one_site();
    assert!(result.is_err());
}
