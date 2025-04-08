use html5ever::{namespace_url, ns, LocalName, QualName};
use schemars::JsonSchema;
use scraper::{Html, Node, Selector};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CssTagReplace {
    #[schemars(with = "String")]
    pub selector: Selector,

    #[schemars(with = "String")]
    pub replace: LocalName,
}

impl CssTagReplace {
    pub fn apply(&self, html: &str) -> String {
        let mut html = Html::parse_document(html);
        let selected = html
            .select(&self.selector)
            .map(|element| {
                (
                    element.id(),
                    element.value().clone(),
                    element
                        .children()
                        .map(|child| child.id())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        for (node_id, mut element, children) in selected {
            element.name = QualName::new(None, ns!(html), self.replace.clone());

            let mut node = html
                .tree
                .get_mut(node_id)
                .expect("Element ID should exist as it was just taken from the given HTML");

            let mut new_node = node.insert_after(Node::Element(element));
            for child in children {
                new_node.append_id(child);
            }

            node.detach();
        }

        html.html()
    }
}

#[cfg(test)]
#[track_caller]
fn case<TAG: Into<LocalName>>(selectors: &str, replace: TAG, html: &str, expected: &str) {
    let result = CssTagReplace {
        selector: Selector::parse(selectors).unwrap(),
        replace: replace.into(),
    }
    .apply(html);
    assert_eq!(result, expected);
}

#[test]
fn only_tag() {
    let html = "<body><h1>Hello</h1>World<h3>Foo</h3>Bar</body>";
    let expected = "<html><head></head><body><h1>Hello</h1>World<h2>Foo</h2>Bar</body></html>";
    case("h3", "h2", html, expected);
}

#[test]
fn keeps_attributes() {
    let html = r#"<body><h2 class="green">Hello</h2>World</body>"#;
    let expected = r#"<html><head></head><body><h1 class="green">Hello</h1>World</body></html>"#;
    case("h2", "h1", html, expected);
}
