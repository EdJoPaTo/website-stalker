use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::tendril::TendrilSink;
use html5ever::QualName;
use url::Url;

struct HtmlAbsLinkSerializer<Wr: Write> {
    serializer: HtmlSerializer<Wr>,
    base_url: Url,
}

impl<Wr: Write> HtmlAbsLinkSerializer<Wr> {
    fn new(writer: Wr, opts: SerializeOpts, base_url: Url) -> Self {
        Self {
            serializer: HtmlSerializer::new(writer, opts),
            base_url,
        }
    }
}

impl<Wr: Write> Serializer for HtmlAbsLinkSerializer<Wr> {
    fn start_elem<'a, AttrIter>(&mut self, name: QualName, attrs: AttrIter) -> std::io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        let mut result_attrs = Vec::new();
        for (key, value) in attrs {
            let value = if &key.local == "href" || &key.local == "src" {
                self.base_url
                    .join(value)
                    .map_err(|err| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            anyhow::anyhow!("failed to parse url {value} {err}"),
                        )
                    })?
                    .to_string()
            } else {
                value.to_string()
            };
            result_attrs.push((key, value));
        }
        self.serializer
            .start_elem(name, result_attrs.iter().map(|o| (o.0, o.1.as_str())))
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
    let doc = kuchikiki::parse_html().one(html);
    let result = serialize(&doc, url)?;
    Ok(result)
}

fn serialize<T: Serialize>(node: &T, base_url: &Url) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlAbsLinkSerializer::new(&mut buf, opts, base_url.clone());
    let opts = SerializeOpts::default();
    node.serialize(&mut ser, opts.traversal_scope)?;

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
#[should_panic = "failed to parse url /// empty host"]
fn garbage_results_in_error() {
    let base_url = Url::parse("https://edjopato.de/index.html").unwrap();
    let ugly = r#"<html><body>Just a <a href="///">test</a></body></html>"#;
    canonicalize(&base_url, ugly).unwrap();
}
