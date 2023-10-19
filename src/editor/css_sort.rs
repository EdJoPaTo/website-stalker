use serde::{Deserialize, Serialize};
use url::Url;

use super::Editor;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CssSort {
    pub selector: String,

    #[serde(default)]
    pub reverse: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort_by: Vec<Editor>,
}

struct Internal {
    selector: scraper::Selector,
}

impl CssSort {
    fn parse(&self) -> anyhow::Result<Internal> {
        let scrape_selector = scraper::Selector::parse(&self.selector)
            .map_err(|err| anyhow::anyhow!("selector ({}) parse error: {err:?}", self.selector))?;

        Ok(Internal {
            selector: scrape_selector,
        })
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse()?;
        Editor::many_valid(&self.sort_by)?;
        Ok(())
    }

    pub fn apply(&self, url: &Url, html: &str) -> anyhow::Result<String> {
        let internal = self.parse()?;

        let parsed_html = scraper::Html::parse_document(html);
        let mut selected = parsed_html.select(&internal.selector).collect::<Vec<_>>();

        if selected.is_empty() {
            anyhow::bail!("selector ({}) selected nothing", self.selector);
        }

        selected.sort_by_cached_key(|item| {
            let mut content = super::Content {
                extension: Some("html"),
                text: item.html(),
            };
            for editor in &self.sort_by {
                if let Ok(c) = editor.apply(url, &content) {
                    content = c;
                } else {
                    break;
                }
            }
            content.text
        });

        if self.reverse {
            selected.reverse();
        }

        Ok(selected
            .iter()
            .map(scraper::ElementRef::html)
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

#[test]
fn valid() {
    let s = CssSort {
        selector: "ul".to_string(),
        sort_by: Vec::new(),
        reverse: false,
    };
    let result = dbg!(s.is_valid());
    assert!(result.is_ok());
}

#[test]
#[should_panic = "parse error"]
fn invalid() {
    CssSort {
        selector: ".".to_string(),
        sort_by: Vec::new(),
        reverse: false,
    }
    .is_valid()
    .unwrap();
}

#[test]
fn simple_example() {
    let url = Url::parse("https://edjopato.de/").unwrap();
    let input = r"<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = r"<p>A</p>
<p>B</p>
<p>C</p>";
    let s = CssSort {
        selector: "p".to_string(),
        sort_by: Vec::new(),
        reverse: false,
    };
    let html = s.apply(&url, input).unwrap();
    assert_eq!(html, expected);
}

#[test]
fn simple_example_reverse() {
    let url = Url::parse("https://edjopato.de/").unwrap();
    let input = r"<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = r"<p>C</p>
<p>B</p>
<p>A</p>";
    let s = CssSort {
        selector: "p".to_string(),
        sort_by: Vec::new(),
        reverse: true,
    };
    let html = s.apply(&url, input).unwrap();
    assert_eq!(html, expected);
}

#[test]
fn sort_by_example() {
    let url = Url::parse("https://edjopato.de/").unwrap();
    let input = r#"<html><head></head><body>
<article><h3>A</h3><a id="B">Bla</a></article>
<article><h3>B</h3><a id="A">Bla</a></article>
</body></html>"#;
    let expected = r#"<article><h3>B</h3><a id="A">Bla</a></article>
<article><h3>A</h3><a id="B">Bla</a></article>"#;
    let s = CssSort {
        selector: "article".to_string(),
        sort_by: vec![Editor::CssSelect(super::css_selector::CssSelector(
            "a".to_string(),
        ))],
        reverse: false,
    };
    let html = s.apply(&url, input).unwrap();
    assert_eq!(html, expected);
}
