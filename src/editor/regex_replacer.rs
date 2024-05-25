use std::borrow::Cow;

use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RegexReplacer {
    #[serde(deserialize_with = "deserialize_regex")]
    pub pattern: Regex,
    pub replace: String,
}

impl RegexReplacer {
    pub fn replace_all<'t>(&self, text: &'t str) -> Cow<'t, str> {
        self.pattern.replace_all(text, &self.replace)
    }
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<regex::Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    regex::Regex::new(&str).map_err(serde::de::Error::custom)
}

#[test]
fn replaces() {
    let example = RegexReplacer {
        pattern: Regex::new(r"(\w)\w*").unwrap(),
        replace: "$1".to_owned(),
    };
    let result = example.replace_all("Hello world");
    assert_eq!(result, "H w");
}

#[test]
fn replaces_iso() {
    let example = RegexReplacer {
        pattern: Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}").unwrap(),
        replace: "ISO8601".to_owned(),
    };
    let result = example.replace_all("2022-02-14T22:31:00+01:00");
    assert_eq!(result, "ISO8601");
}
