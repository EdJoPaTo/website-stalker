use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CssSort {
    selector: String,
    #[serde(default)]
    sort_by: Option<String>,
    #[serde(default)]
    reverse: bool,
}

struct Internal {
    selector: scraper::Selector,
    sort_by: Option<scraper::Selector>,
}

impl CssSort {
    fn parse(&self) -> anyhow::Result<Internal> {
        let scrape_selector = scraper::Selector::parse(&self.selector)
            .map_err(|err| anyhow::anyhow!("selector ({}) parse error: {err:?}", self.selector))?;

        let scrape_sort_by = if let Some(sort_by) = &self.sort_by {
            Some(
                scraper::Selector::parse(sort_by)
                    .map_err(|err| anyhow::anyhow!("sort_by ({sort_by}) parse error: {err:?}"))?,
            )
        } else {
            None
        };

        Ok(Internal {
            selector: scrape_selector,
            sort_by: scrape_sort_by,
        })
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse()?;
        Ok(())
    }

    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let internal = self.parse()?;

        let parsed_html = scraper::Html::parse_document(html);
        let mut selected = parsed_html.select(&internal.selector).collect::<Vec<_>>();

        if selected.is_empty() {
            anyhow::bail!("selector ({}) selected nothing", self.selector);
        }

        selected.sort_by_cached_key(|o| {
            internal.sort_by.as_ref().map_or_else(
                || o.html(),
                |by| o.select(by).map(|i| i.html()).collect::<String>(),
            )
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
        sort_by: None,
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
        sort_by: None,
        reverse: false,
    }
    .is_valid()
    .unwrap();
}

#[test]
fn simple_example() {
    let input = r"<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = r"<p>A</p>
<p>B</p>
<p>C</p>";
    let s = CssSort {
        selector: "p".to_string(),
        sort_by: None,
        reverse: false,
    };
    let html = s.apply(input).unwrap();
    assert_eq!(html, expected);
}

#[test]
fn simple_example_reverse() {
    let input = r"<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = r"<p>C</p>
<p>B</p>
<p>A</p>";
    let s = CssSort {
        selector: "p".to_string(),
        sort_by: None,
        reverse: true,
    };
    let html = s.apply(input).unwrap();
    assert_eq!(html, expected);
}

#[test]
fn sort_by_example() {
    let input = r#"<html><head></head><body>
<article><h3>A</h3><a id="B">Bla</a></article>
<article><h3>B</h3><a id="A">Bla</a></article>
</body></html>"#;
    let expected = r#"<article><h3>B</h3><a id="A">Bla</a></article>
<article><h3>A</h3><a id="B">Bla</a></article>"#;
    let s = CssSort {
        selector: "article".to_string(),
        sort_by: Some("a".to_string()),
        reverse: false,
    };
    let html = s.apply(input).unwrap();
    assert_eq!(html, expected);
}
