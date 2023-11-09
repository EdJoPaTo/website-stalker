use scraper::Selector;

pub fn apply(selector: &Selector, html: &str) -> anyhow::Result<String> {
    let parsed_html = scraper::Html::parse_document(html);
    let selected = parsed_html
        .select(selector)
        .map(|o| o.html())
        .collect::<Vec<_>>();
    anyhow::ensure!(!selected.is_empty(), "selected nothing");
    Ok(selected.join("\n"))
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn selects_classes_a() {
    let selector = Selector::parse(".a").unwrap();
    let html = apply(&selector, EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="a"><p>A</p></div>"#);
}

#[test]
fn selects_classes_b() {
    let selector = Selector::parse(".b").unwrap();
    let html = apply(&selector, EXAMPLE_HTML).unwrap();
    assert_eq!(html, r#"<div class="b">B</div>"#);
}

#[test]
fn selects_tag() {
    let selector = Selector::parse("p").unwrap();
    let html = apply(&selector, EXAMPLE_HTML).unwrap();
    assert_eq!(html, "<p>A</p>");
}

#[test]
#[should_panic = "selected nothing"]
fn select_not_found() {
    let selector = Selector::parse("p").unwrap();
    apply(&selector, "<html><head></head><body>test</body></html>").unwrap();
}
