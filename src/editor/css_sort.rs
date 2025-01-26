use std::collections::HashMap;

use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use url::Url;

use super::Editor;
use crate::logger;

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
    pub fn apply(&self, url: &Url, html: &str) -> String {
        let mut html = Html::parse_document(html);
        let selected = html.select(&self.selector).collect::<Vec<_>>();

        let mut grouped_by_parent: HashMap<_, Vec<ElementRef>> = HashMap::new();
        for element in selected {
            if let Some(key) = element.parent().map(|parent| parent.id()) {
                grouped_by_parent.entry(key).or_default().push(element);
            }
        }

        if grouped_by_parent.is_empty() {
            logger::warn(&format!("css_sort selector selected nothing to sort {url}"));
        }

        // Get the order of the elements as ids
        // This removes the reference to html allowing to take mut references later on
        let sorted = grouped_by_parent
            .into_iter()
            .map(|(parent, mut elements)| {
                elements.sort_by_cached_key(|element| self.get_sort_key_from_element(url, element));
                if self.reverse {
                    elements.reverse();
                }
                let elements = elements
                    .iter()
                    .map(|element| element.id())
                    .collect::<Vec<_>>();
                (parent, elements)
            })
            .collect::<HashMap<_, _>>();

        for (parent, sorted) in sorted {
            for id in &sorted {
                html.tree.get_mut(*id).unwrap().detach();
            }

            // Insert them at the beginning of the parents children
            // This destroys the order with the other elements in there but its way simpler to do for now
            let mut parent_mut = html.tree.get_mut(parent).unwrap();
            for id in sorted.into_iter().rev() {
                parent_mut.prepend_id(id);
            }
        }

        html.html()
    }

    fn get_sort_key_from_element(&self, url: &Url, element: &ElementRef) -> String {
        let content = super::Content {
            extension: Some("html"),
            text: element.html(),
        };
        Editor::apply_many(&self.sort_by, url, content).map_or_else(
            |error| {
                logger::error(&format!("css_sort sort_by failed {error:#}"));
                String::new()
            },
            |content| content.text,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn case(css_sort: &CssSort, input: &str, expected: &str) {
        const PREFIX: &str = "<html><head></head><body>";
        const SUFFIX: &str = "</body></html>";

        let url = Url::parse("https://edjopato.de/").unwrap();
        let html = css_sort.apply(&url, input);

        assert!(html.starts_with(PREFIX));
        assert!(html.ends_with(SUFFIX));
        let end_index = html.len() - SUFFIX.len();
        let html = html.get(PREFIX.len()..end_index).unwrap();

        assert_eq!(html, expected);
    }

    #[test]
    fn simple_example() {
        let input = "<p>A</p><p>C</p><p>B</p>";
        let expected = "<p>A</p><p>B</p><p>C</p>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn reverse() {
        let input = "<p>A</p><p>C</p><p>B</p>";
        let expected = "<p>C</p><p>B</p><p>A</p>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: true,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sort_by() {
        let input = r#"<article><h3>A</h3><a id="Y">Bla</a></article><article><h3>B</h3><a id="X">Bla</a></article>"#;
        let expected = r#"<article><h3>B</h3><a id="X">Bla</a></article><article><h3>A</h3><a id="Y">Bla</a></article>"#;
        let sort_by = CssSort {
            selector: Selector::parse("article").unwrap(),
            sort_by: vec![Editor::CssSelect(Selector::parse("a").unwrap())],
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sort_by_same_key_keeps_order() {
        let input = r#"<article><h3>C</h3><a id="X">Bla</a></article><article><h3>A</h3><a id="X">Bla</a></article>"#;
        let expected = r#"<article><h3>C</h3><a id="X">Bla</a></article><article><h3>A</h3><a id="X">Bla</a></article>"#;
        let sort_by = CssSort {
            selector: Selector::parse("article").unwrap(),
            sort_by: vec![Editor::CssSelect(Selector::parse("a").unwrap())],
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sorting_toplevel_keeps_children_unsorted() {
        let input = "<div><p>D</p><p>A</p></div><div><p>C</p><p>B</p></div>";
        let expected = "<div><p>C</p><p>B</p></div><div><p>D</p><p>A</p></div>";
        let sort_by = CssSort {
            selector: Selector::parse("div").unwrap(),
            sort_by: Vec::new(),
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    #[test]
    fn sorting_bottomlevel_keeps_parents_unsorted() {
        let input = "<div><p>D</p><p>A</p></div><div><p>C</p><p>B</p></div>";
        let expected = "<div><p>A</p><p>D</p></div><div><p>B</p><p>C</p></div>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: false,
        };
        case(&sort_by, input, expected);
    }

    /// Documents current sorting order when other elements are there.
    /// Needs to be adapted when sorting order is improved.
    #[test]
    fn sort_with_other_elements() {
        let input = "<div>1</div><p>A</p><img><p>B</p>";
        let expected = "<p>A</p><p>B</p><div>1</div><img>";
        let sort_by = CssSort {
            selector: Selector::parse("p").unwrap(),
            sort_by: Vec::new(),
            reverse: false,
        };
        case(&sort_by, input, expected);
    }
}
