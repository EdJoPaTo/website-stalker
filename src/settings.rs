use config::ConfigError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub enum Site {
    Html(HtmlSite),
    Utf8(Utf8Site),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HtmlSite {
    pub url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Utf8Site {
    pub url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub from: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    pub sites: Vec<Site>,
}

impl Settings {
    pub fn example() -> Self {
        Self {
            from: "my-email-address".to_string(),
            user_agent: None,
            sites: vec![
                Site::Html(HtmlSite {
                    url: Url::parse("https://edjopato.de/post/").unwrap(),
                }),
                Site::Utf8(Utf8Site {
                    url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
                }),
            ],
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
        let from = &self.from;
        if !from.contains('@') || !from.contains('.') {
            return Err(ConfigError::Message(format!(
                "from doesnt look like an email address: {}",
                from
            )));
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
