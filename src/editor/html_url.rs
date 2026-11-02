use url::Url;

use crate::logger;

pub fn canonicalize(base_url: &Url, html: &str) -> String {
    let mut html = scraper::Html::parse_document(html);
    for node in html.tree.values_mut() {
        let scraper::Node::Element(element) = node else {
            continue;
        };
        for (key, value) in &mut element.attrs {
            if &key.local == "href" || &key.local == "src" {
                match base_url.join(value) {
                    Ok(url) => {
                        value.clear();
                        value.push_slice(url.as_ref());
                    }
                    Err(error) => logger::warn(&format!(
                        "{base_url} html_url_canonicalize could not parse url {value}: {error}"
                    )),
                }
            }
        }
    }
    html.html()
}

#[test]
fn works_with_a_links() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a>test</a> <a href="/post/">link</a></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head></head><body>Just a <a>test</a> <a href="https://edjopato.de/post/">link</a></body></html>"#
    );
}

#[test]
fn works_with_img() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <img src="/assets/cheese.svg"> test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head></head><body>Just a <img src="https://edjopato.de/assets/cheese.svg"> test</body></html>"#
    );
}

#[test]
fn works_with_stylesheet() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><head><link href="/index.css" rel="stylesheet"></head><body>Just a test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head><link href="https://edjopato.de/index.css" rel="stylesheet"></head><body>Just a test</body></html>"#
    );
}

#[test]
fn works_with_script() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly =
        r#"<html><head><script src="/updatebg.js"></script></head><body>Just a test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head><script src="https://edjopato.de/updatebg.js"></script></head><body>Just a test</body></html>"#
    );
}

#[test]
fn works_with_already_absolute_url() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a href="https://3t0.de/">test</a></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head></head><body>Just a <a href="https://3t0.de/">test</a></body></html>"#
    );
}

#[test]
fn garbage_is_skipped() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a href="///">test</a> that <a href="https://edjopato.de">works</a></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head></head><body>Just a <a href="///">test</a> that <a href="https://edjopato.de/">works</a></body></html>"#
    );
}

#[test]
fn doesnt_touch_noscript() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly =
        r#"<html><body>Just a test <noscript><a href="/post/">link</a></noscript></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly),
        r#"<html><head></head><body>Just a test <noscript><a href="/post/">link</a></noscript></body></html>"#
    );
}

#[test]
fn unrelated_not_yet_escaped_text_is_escaped() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = "<html><body>Go -> there & save the world while already escaped &amp; keeps escaped</body></html>";
    assert_eq!(
        canonicalize(&base_url, ugly),
        "<html><head></head><body>Go -&gt; there &amp; save the world while already escaped &amp; keeps escaped</body></html>"
    );
}

#[test]
fn style_tag_content_is_not_escaped() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = "<html><body><style>body > a { margin: 0; }</style></body></html>";
    assert_eq!(
        canonicalize(&base_url, ugly),
        "<html><head></head><body><style>body > a { margin: 0; }</style></body></html>"
    );
}
