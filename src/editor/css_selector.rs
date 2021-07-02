use serde::{Deserialize, Serialize};

// TODO: deserialize from string or struct

#[derive(Debug, Serialize, Deserialize)]
pub struct CssSelector {
    pub selector: String,

    #[serde(skip_serializing_if = "is_default")]
    pub remove: bool,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl CssSelector {
    fn parse(&self) -> anyhow::Result<scraper::Selector> {
        let scrape_selector = scraper::Selector::parse(&self.selector).map_err(|err| {
            anyhow::anyhow!("css selector ({}) parse error: {:?}", self.selector, err)
        })?;

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

        if self.remove {
            let mut html = parsed_html.root_element().html();
            for s in selected {
                html = html.replace(&s, "");
            }
            Ok(html)
        } else {
            if selected.is_empty() {
                return Err(anyhow::anyhow!(
                    "css_selector ({}) selected nothing",
                    self.selector
                ));
            }
            Ok(selected.join("\n"))
        }
    }
}

#[test]
fn valid() {
    let s = CssSelector {
        selector: "body".to_string(),
        remove: false,
    };
    let result = s.is_valid();
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let s = CssSelector {
        selector: ".".to_string(),
        remove: false,
    };
    let result = s.is_valid();
    println!("{:?}", result);
    assert!(result.is_err());
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn selects_classes_a() {
    let selector = CssSelector {
        selector: ".a".to_string(),
        remove: false,
    };
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="a"><p>A</p></div>"#);
}

#[test]
fn selects_classes_b() {
    let selector = CssSelector {
        selector: ".b".to_string(),
        remove: false,
    };
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="b">B</div>"#);
}

#[test]
fn selects_tag() {
    let selector = CssSelector {
        selector: "p".to_string(),
        remove: false,
    };
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<p>A</p>"#);
}

#[test]
fn removes_tag() {
    let selector = CssSelector {
        selector: "p".to_string(),
        remove: true,
    };
    let html = selector.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b">B</div></body></html>"#
    );
}
