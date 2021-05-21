use regex::Regex;

use crate::http::Http;

pub struct HuntOutput {
    pub content: String,
    pub filename: String,
}

pub trait Huntable {
    fn hunt(&self, http_agent: &Http) -> anyhow::Result<HuntOutput>;
}

pub fn format_url_as_filename(url: &url::Url, extension: &str) -> String {
    let re = Regex::new("[^a-zA-Z\\d]+").unwrap();
    let only_ascii = re.replace_all(url.as_str(), "-");
    let trimmed = only_ascii.trim_matches('-');
    format!("{}.{}", trimmed, extension)
}
