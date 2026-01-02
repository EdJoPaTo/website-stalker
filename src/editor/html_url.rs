use std::io::Write;

use html5ever::QualName;
use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize as _, SerializeOpts, Serializer};
use scraper::Html;
use url::Url;

use crate::logger;

struct HtmlAbsLinkSerializer<'url, Wr: Write> {
    serializer: HtmlSerializer<Wr>,
    base_url: &'url Url,
}

impl<'url, Wr: Write> HtmlAbsLinkSerializer<'url, Wr> {
    fn new(writer: Wr, opts: SerializeOpts, base_url: &'url Url) -> Self {
        Self {
            serializer: HtmlSerializer::new(writer, opts),
            base_url,
        }
    }
}

impl<Wr: Write> Serializer for HtmlAbsLinkSerializer<'_, Wr> {
    fn start_elem<'a, AttrIter>(&mut self, name: QualName, attrs: AttrIter) -> std::io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        let mut result_attrs = Vec::new();
        for (key, value) in attrs {
            let value = if &key.local == "href" || &key.local == "src" {
                match self.base_url.join(value) {
                    Ok(url) => url.to_string(),
                    Err(error) => {
                        logger::warn(&format!(
                            "{} html_url_canonicalize could not parse url {value}: {error}",
                            self.base_url
                        ));
                        value.to_owned()
                    }
                }
            } else {
                value.to_owned()
            };
            result_attrs.push((key, value));
        }
        self.serializer.start_elem(
            name,
            result_attrs
                .iter()
                .map(|(attribute_name, value)| (*attribute_name, value.as_str())),
        )
    }

    fn end_elem(&mut self, name: QualName) -> std::io::Result<()> {
        self.serializer.end_elem(name)
    }

    fn write_text(&mut self, text: &str) -> std::io::Result<()> {
        write!(self.serializer.writer, "{text}")
    }

    fn write_comment(&mut self, text: &str) -> std::io::Result<()> {
        self.serializer.write_comment(text)
    }

    fn write_doctype(&mut self, name: &str) -> std::io::Result<()> {
        self.serializer.write_doctype(name)
    }

    fn write_processing_instruction(&mut self, target: &str, data: &str) -> std::io::Result<()> {
        self.serializer.write_processing_instruction(target, data)
    }
}

pub fn canonicalize(url: &Url, html: &str) -> anyhow::Result<String> {
    reserialize(html, url)
}

fn reserialize(html: &str, base_url: &Url) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlAbsLinkSerializer::new(&mut buf, opts, base_url);
    let opts = SerializeOpts::default();
    Html::parse_document(html).serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

#[test]
fn works_with_a_links() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a>test</a> <a href="/post/">link</a></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head></head><body>Just a <a>test</a> <a href="https://edjopato.de/post/">link</a></body></html>"#
    );
}

#[test]
fn works_with_img() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <img src="/assets/cheese.svg"> test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head></head><body>Just a <img src="https://edjopato.de/assets/cheese.svg"> test</body></html>"#
    );
}

#[test]
fn works_with_stylesheet() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><head><link href="/index.css" rel="stylesheet"></head><body>Just a test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head><link href="https://edjopato.de/index.css" rel="stylesheet"></head><body>Just a test</body></html>"#
    );
}

#[test]
fn works_with_script() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly =
        r#"<html><head><script src="/updatebg.js"></script></head><body>Just a test</body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head><script src="https://edjopato.de/updatebg.js"></script></head><body>Just a test</body></html>"#
    );
}

#[test]
fn works_with_already_absolute_url() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a href="https://3t0.de/">test</a></body></html>"#;
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head></head><body>Just a <a href="https://3t0.de/">test</a></body></html>"#
    );
}

#[test]
fn garbage_is_skipped() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a href="///">test</a> that <a href="https://edjopato.de">works</a></body></html>"#;
    canonicalize(&base_url, ugly).unwrap();
    assert_eq!(
        canonicalize(&base_url, ugly).unwrap(),
        r#"<html><head></head><body>Just a <a href="///">test</a> that <a href="https://edjopato.de/">works</a></body></html>"#
    );
}
