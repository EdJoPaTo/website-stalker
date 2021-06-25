#[derive(Debug)]
pub struct CssSelector {
    plain: String,
    is_removal: bool,
    scrape_selector: scraper::Selector,
}

impl CssSelector {
    pub fn parse(selector: &str) -> anyhow::Result<Self> {
        let is_removal = selector.starts_with('!');
        let plain = selector.trim_start_matches('!').trim().to_string();

        let scrape_selector = scraper::Selector::parse(&plain)
            .map_err(|err| anyhow::anyhow!("css selector ({}) parse error: {:?}", selector, err))?;

        Ok(Self {
            plain,
            is_removal,
            scrape_selector,
        })
    }

    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let parsed_html = scraper::Html::parse_document(html);
        let selected = parsed_html
            .select(&self.scrape_selector)
            .map(|o| o.html())
            .collect::<Vec<_>>();

        if self.is_removal {
            let mut html = parsed_html.root_element().html();
            for s in selected {
                html = html.replace(&s, "");
            }
            Ok(html)
        } else {
            if selected.is_empty() {
                return Err(anyhow::anyhow!(
                    "css_selector ({}) selected nothing",
                    self.plain
                ));
            }
            Ok(selected.join("\n"))
        }
    }
}

impl serde::Serialize for CssSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_removal {
            serializer.serialize_str(&format!("!{}", self.plain))
        } else {
            serializer.serialize_str(&self.plain)
        }
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
fn valid() {
    let s = "body";
    let result = CssSelector::parse(s);
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let s = ".";
    let result = CssSelector::parse(s);
    println!("{:?}", result);
    assert!(result.is_err());
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn selects_classes_a() {
    let selector = CssSelector::parse(".a").unwrap();
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="a"><p>A</p></div>"#);
}

#[test]
fn selects_classes_b() {
    let selector = CssSelector::parse(".b").unwrap();
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="b">B</div>"#);
}

#[test]
fn selects_tag() {
    let selector = CssSelector::parse("p").unwrap();
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<p>A</p>"#);
}

#[test]
fn removes_tag() {
    let selector = CssSelector::parse("!p").unwrap();
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b">B</div></body></html>"#
    );
}
