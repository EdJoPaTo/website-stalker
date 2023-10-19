use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CssSelector(pub String);

impl CssSelector {
    fn parse(&self) -> anyhow::Result<scraper::Selector> {
        let scrape_selector = scraper::Selector::parse(&self.0)
            .map_err(|err| anyhow::anyhow!("({}) parse error: {err:?}", self.0))?;

        Ok(scrape_selector)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse()?;
        Ok(())
    }

    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let parsed_html = scraper::Html::parse_document(html);
        let selected = parsed_html
            .select(&self.parse()?)
            .map(|o| o.html())
            .collect::<Vec<_>>();

        if selected.is_empty() {
            anyhow::bail!("selected nothing ({})", self.0);
        }
        Ok(selected.join("\n"))
    }
}

#[test]
fn valid() {
    let s = CssSelector("body".to_string());
    let result = dbg!(s.is_valid());
    assert!(result.is_ok());
}

#[test]
#[should_panic = "parse error"]
fn invalid() {
    CssSelector(".".to_string()).is_valid().unwrap();
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn selects_classes_a() {
    let selector = CssSelector(".a".to_string());
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="a"><p>A</p></div>"#);
}

#[test]
fn selects_classes_b() {
    let selector = CssSelector(".b".to_string());
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="b">B</div>"#);
}

#[test]
fn selects_tag() {
    let selector = CssSelector("p".to_string());
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r"<p>A</p>");
}

#[test]
#[should_panic = "selected nothing"]
fn select_not_found() {
    CssSelector("p".to_string())
        .apply(r"<html><head></head><body>test</body></html>")
        .unwrap();
}
