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

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn case(css_sort: &CssSort, input: &str, expected: &str) {
        let url = Url::parse("https://edjopato.de/").unwrap();
        let html = css_sort.apply(&url, input).unwrap();
        assert_eq!(html, expected);
    }

    #[test]
    fn simple_example() {
        let input = "<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
        let expected = "<p>A</p>
<p>B</p>
<p>C</p>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn reverse() {
        let input = "<html><head></head><body><p>A</p><p>C</p><p>B</p></body></html>";
        let expected = "<p>C</p>
<p>B</p>
<p>A</p>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: true,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sort_by() {
        let input = r#"<html><head></head><body>
<article><h3>A</h3><a id="Y">Bla</a></article>
<article><h3>B</h3><a id="X">Bla</a></article>
</body></html>"#;
        let expected = r#"<article><h3>B</h3><a id="X">Bla</a></article>
<article><h3>A</h3><a id="Y">Bla</a></article>"#;
        let sort_by = CssSort {
            selector: Selector::parse("article").unwrap(),
            sort_by: vec![Editor::CssSelect(Selector::parse("a").unwrap())],
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sort_by_same_key_keeps_order() {
        let input = r#"<html><head></head><body>
<article><h3>C</h3><a id="X">Bla</a></article>
<article><h3>A</h3><a id="X">Bla</a></article>
</body></html>"#;
        let expected = r#"<article><h3>C</h3><a id="X">Bla</a></article>
<article><h3>A</h3><a id="X">Bla</a></article>"#;
        let sort_by = CssSort {
            selector: Selector::parse("article").unwrap(),
            sort_by: vec![Editor::CssSelect(Selector::parse("a").unwrap())],
            reverse: false,
        };
        case(&sort_by, input, expected);
    }
}
