use std::borrow::Cow;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegexReplacer {
    pub pattern: String,
    pub replace: String,
}

impl RegexReplacer {
    pub fn is_valid(&self) -> Result<(), regex::Error> {
        Regex::new(&self.pattern)?;
        Ok(())
    }

    pub fn replace_all<'t>(&self, text: &'t str) -> Result<Cow<'t, str>, regex::Error> {
        let re = Regex::new(&self.pattern)?;
        let result = re.replace_all(text, &self.replace);
        Ok(result)
    }
}

#[test]
fn is_valid_true_example() {
    let example = RegexReplacer {
        pattern: r#"(class)="[^"]+"#.to_string(),
        replace: "$1".to_string(),
    };
    let result = dbg!(example.is_valid());
    assert!(result.is_ok());
}

#[test]
#[should_panic = "unclosed group"]
fn is_valid_false_example() {
    let example = RegexReplacer {
        pattern: "(class".to_string(),
        replace: String::new(),
    };
    example.is_valid().unwrap();
}

#[test]
fn replaces() -> anyhow::Result<()> {
    let example = RegexReplacer {
        pattern: r#"(\w)\w*"#.to_string(),
        replace: "$1".to_string(),
    };
    let result = example.replace_all("Hello world")?;
    assert_eq!(result, "H w");
    Ok(())
}

#[test]
fn replaces_iso() -> anyhow::Result<()> {
    let example = RegexReplacer {
        pattern: r#"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}"#.to_string(),
        replace: "ISO8601".to_string(),
    };
    let result = example.replace_all("2022-02-14T22:31:00+01:00")?;
    assert_eq!(result, "ISO8601");
    Ok(())
}
