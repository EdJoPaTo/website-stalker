pub fn sanitize(html: &str) -> String {
    ammonia::clean(html)
}

#[test]
fn simple() {
    let ugly = "<html><body>Just a <div>test</div></body></html>";
    assert_eq!(sanitize(ugly), "Just a <div>test</div>");
}

#[test]
fn style_gone() {
    let ugly = r#"<html><body><div style="--a: 42;">test</div></body></html>"#;
    assert_eq!(sanitize(ugly), "<div>test</div>");
}

#[test]
fn class_gone() {
    let ugly = r#"<html><body><div class="a">test</div></body></html>"#;
    assert_eq!(sanitize(ugly), "<div>test</div>");
}

#[test]
fn link_still_useful() {
    let ugly = r#"<a class="external" href="https://edjopato.de" target="_blank" rel="noopener noreferrer">test</a>"#;
    assert_eq!(
        sanitize(ugly),
        r#"<a href="https://edjopato.de" rel="noopener noreferrer">test</a>"#
    );
}
