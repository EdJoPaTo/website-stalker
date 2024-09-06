use scraper::Selector;

pub fn apply(selector: &Selector, html: &str) -> String {
    let mut html = scraper::Html::parse_document(html);
    let selected = html
        .select(selector)
        .map(|element| element.id())
        .collect::<Vec<_>>();
    for id in selected {
        let children = html
            .tree
            .get(id)
            .map(|parent| {
                parent
                    .children()
                    .map(|child| child.id())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if let Some(mut node_mut) = html.tree.get_mut(id) {
            for id in children {
                node_mut.insert_id_after(id);
            }
            node_mut.detach();
        }
    }
    html.html()
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[cfg(test)]
#[track_caller]
fn case(selector: &str, expected: &str) {
    let selector = Selector::parse(selector).unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(html, expected);
}

#[test]
fn nothing_selected_changes_nothing() {
    case("span", EXAMPLE_HTML);
}

#[test]
fn simple() {
    case(
        "p",
        r#"<html><head></head><body><div class="a">A</div><div class="b">B</div></body></html>"#,
    );
}

#[test]
fn multiple() {
    case(
        ".b, p",
        r#"<html><head></head><body><div class="a">A</div>B</body></html>"#,
    );
}

#[test]
fn multiple_selectors_inside_each_other_work() {
    let expected = r#"<html><head></head><body>A<div class="b">B</div></body></html>"#;
    case(".a, p", expected);
    case("p, .a", expected);
}
