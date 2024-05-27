use scraper::Selector;
use serde::Deserialize;
use url::Url;

use super::Editor;

#[derive(Debug, Clone, Deserialize)]
pub struct CssSort {
    #[serde(deserialize_with = "super::deserialize_selector")]
    pub selector: Selector,

    #[serde(default)]
    pub reverse: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort_by: Vec<Editor>,
}

impl CssSort {
    pub fn apply(&self, url: &Url, html: &str) -> anyhow::Result<String> {
        let parsed_html = scraper::Html::parse_document(html);
        let mut selected = parsed_html.select(&self.selector).collect::<Vec<_>>();

        if selected.is_empty() {
            anyhow::bail!("selected nothing");
        }

        selected.sort_by_cached_key(|item| {
            let mut content = super::Content {
                extension: Some("html"),
                text: item.html(),
            };
            for editor in &self.sort_by {
                if let Ok(inner) = editor.apply(url, content) {
                    content = inner;
                } else {
                    return String::new();
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
fn simple_example() {
    let url = Url::parse("https://edjopato.de/").unwrap();
    let input = "<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = "<p>A</p>
<p>B</p>
<p>C</p>";
    let example = CssSort {
        selector: Selector::parse("p").unwrap(),
        sort_by: Vec::new(),
        reverse: false,
    };
    let html = example.apply(&url, input).unwrap();
    assert_eq!(html, expected);
}

#[test]
fn simple_example_reverse() {
    let url = Url::parse("https://edjopato.de/").unwrap();
    let input = "<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
    let expected = "<p>C</p>
<p>B</p>
<p>A</p>";
    let example = CssSort {
        selector: Selector::parse("p").unwrap(),
        sort_by: Vec::new(),
        reverse: true,
    };
    let html = example.apply(&url, input).unwrap();
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
    let example = CssSort {
        selector: Selector::parse("article").unwrap(),
        sort_by: vec![Editor::CssSelect(Selector::parse("a").unwrap())],
        reverse: false,
    };
    let html = example.apply(&url, input).unwrap();
    assert_eq!(html, expected);
}
