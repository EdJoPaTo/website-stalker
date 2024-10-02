use scraper::Selector;

pub fn apply(selector: &Selector, html: &str) -> String {
    let mut html = scraper::Html::parse_document(html);
    let selected = html
        .select(selector)
        .map(|element| element.id())
        .collect::<Vec<_>>();
    for id in selected {
        if let Some(mut node_mut) = html.tree.get_mut(id) {
            node_mut.detach();
        }
    }
    html.html()
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn removes_tag() {
    let selector = Selector::parse("p").unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b">B</div></body></html>"#
    );
}

#[test]
fn nothing_selected_changes_nothing() {
    let selector = Selector::parse("span").unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(html, EXAMPLE_HTML);
}

#[test]
fn multiple_selectors_work() {
    let selector = Selector::parse(".b, p").unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div></body></html>"#
    );
}

#[test]
fn multiple_selectors_inside_each_other_work() {
    let expected = r#"<html><head></head><body><div class="b">B</div></body></html>"#;

    let selector = Selector::parse("p, .a").unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(html, expected);

    let selector = Selector::parse(".a, p").unwrap();
    let html = apply(&selector, EXAMPLE_HTML);
    assert_eq!(html, expected);
}

#[test]
fn multiple_hits_only_remove_exact() {
    let selector = Selector::parse(".a p").unwrap();
    let html = apply(
        &selector,
        r#"<html><head></head><body><div class="a"><p>TEST</p></div><div class="b"><p>TEST</p></div></body></html>"#,
    );
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b"><p>TEST</p></div></body></html>"#
    );
}
