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
    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let mut html = Html::parse_document(html);
        let selected = html
            .select(&self.selector)
            .map(|element| element.id())
            .collect::<Vec<_>>();
        anyhow::ensure!(!selected.is_empty(), "selected nothing");
        for node_id in selected {
            let mut node = html
                .tree
                .get_mut(node_id)
                .expect("Element ID should exist as it was just taken from the given HTML");
            let Node::Element(element) = node.value() else {
                unreachable!("Select only selects elements");
            };
            element.name = QualName::new(None, ns!(html), self.replace.clone());
        }

        Ok(html.html())
    }
}

#[cfg(test)]
#[track_caller]
fn case<TAG: Into<LocalName>>(selectors: &str, replace: TAG, html: &str, expected: &str) {
    let result = CssTagReplace {
        selector: Selector::parse(selectors).unwrap(),
        replace: replace.into(),
    }
    .apply(html)
    .expect("Should select something");
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

#[test]
fn more_specific_selector() {
    let html = "<body><header><div>Headline</div></header><main><div>Something</div></main></body>";
    let expected = "<html><head></head><body><header><div>Headline</div></header><main><p>Something</p></main></body></html>";
    case("main div", "p", html, expected);
}
