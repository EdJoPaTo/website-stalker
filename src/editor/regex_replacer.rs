use std::borrow::Cow;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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
    let result = example.is_valid();
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn is_valid_false_example() {
    let example = RegexReplacer {
        pattern: "(class".to_string(),
        replace: "".to_string(),
    };
    let result = example.is_valid();
    println!("{:?}", result);
    assert!(result.is_err());
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
