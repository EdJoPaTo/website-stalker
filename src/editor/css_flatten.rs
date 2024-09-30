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
                node_mut.insert_id_before(id);
            }
            node_mut.detach();
        }
    }
    html.html()
}

#[cfg(test)]
#[track_caller]
fn case(selector: &str, html: &str, expected: &str) {
    let selector = Selector::parse(selector).unwrap();
    let actual = apply(&selector, html);
    assert_eq!(actual, expected);
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn nothing_selected_changes_nothing() {
    case("span", EXAMPLE_HTML, EXAMPLE_HTML);
}

#[test]
fn simple() {
    let expected =
        r#"<html><head></head><body><div class="a">A</div><div class="b">B</div></body></html>"#;
    case("p", EXAMPLE_HTML, expected);
}

#[test]
fn multiple_selectors() {
    let expected = r#"<html><head></head><body><div class="a">A</div>B</body></html>"#;
    case(".b, p", EXAMPLE_HTML, expected);
}

#[test]
fn multiple_selectors_inside_each_other_work() {
    let expected = r#"<html><head></head><body>A<div class="b">B</div></body></html>"#;
    case(".a, p", EXAMPLE_HTML, expected);
    case("p, .a", EXAMPLE_HTML, expected);
}

#[test]
fn multiple_children() {
    let input =
        "<html><head></head><body><div><p>A</p><p>B</p></div><div><p>C</p></div></body></html>";
    let expected = "<html><head></head><body><p>A</p><p>B</p><p>C</p></body></html>";
    case("div", input, expected);
}

#[test]
fn selector_selects_multiple_depths() {
    let input = r#"<html><head></head><body><div class="outer">
<div class="a"><div class="a-inner"><p>A</p></div></div>
<div class="b"><p>B</p></div>
</div></body></html>"#
        .replace('\n', "");
    let expected = "<html><head></head><body><p>A</p><p>B</p></body></html>";
    case("div", &input, expected);
}

#[test]
fn flattens_local_links_away() {
    let input = r##"<html><head></head><body><a href="#heading"><h1>Heading</h1></a><p>This is a <a href="https://edjopato.de">link</a></p></body></html>"##;
    let expected = r#"<html><head></head><body><h1>Heading</h1><p>This is a <a href="https://edjopato.de">link</a></p></body></html>"#;
    case(r##"a[href^="#"]"##, input, expected);
}
