#[derive(Debug)]
pub struct CssSelector {
    plain: String,
    scrape_selector: scraper::Selector,
}

impl CssSelector {
    pub fn parse(selector: &str) -> anyhow::Result<Self> {
        let scrape_selector = scraper::Selector::parse(selector)
            .map_err(|err| anyhow::anyhow!("css selector ({}) parse error: {:?}", selector, err))?;

        Ok(Self {
            plain: selector.to_string(),
            scrape_selector,
        })
    }

    pub fn select(&self, html: &str) -> Vec<String> {
        let html = scraper::Html::parse_document(html);
        html.select(&self.scrape_selector)
            .map(|o| o.html())
            .collect()
    }
}

impl serde::Serialize for CssSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.plain)
    }
}

impl<'de> serde::Deserialize<'de> for CssSelector {
    fn deserialize<D>(deserializer: D) -> Result<CssSelector, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};
        const EXPECTED: &str = "a string representing a CSS selector";
        struct SelectorVisitor;
        impl<'de> Visitor<'de> for SelectorVisitor {
            type Value = CssSelector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                CssSelector::parse(s)
                    .map_err(|_| Error::invalid_value(Unexpected::Str(s), &EXPECTED))
            }
        }

        deserializer.deserialize_str(SelectorVisitor)
    }
}

impl std::fmt::Display for CssSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.plain)
    }
}

#[test]
fn css_selector_valid() {
    let s = "body";
    let result = CssSelector::parse(s);
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn css_selector_invalid() {
    let s = ".";
    let result = CssSelector::parse(s);
    println!("{:?}", result);
    assert!(result.is_err());
}
