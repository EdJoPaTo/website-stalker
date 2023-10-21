use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::tendril::TendrilSink;
use html5ever::QualName;
use lazy_regex::{lazy_regex, Lazy, Regex};

struct HtmlTextSerializer<Wr: Write> {
    serializer: HtmlSerializer<Wr>,
}

impl<Wr: Write> HtmlTextSerializer<Wr> {
    fn new(writer: Wr, opts: SerializeOpts) -> Self {
        Self {
            serializer: HtmlSerializer::new(writer, opts),
        }
    }
}

impl<Wr: Write> Serializer for HtmlTextSerializer<Wr> {
    fn start_elem<'a, AttrIter>(&mut self, _name: QualName, _attrs: AttrIter) -> std::io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        Ok(())
    }

    fn end_elem(&mut self, _name: QualName) -> std::io::Result<()> {
        Ok(())
    }

    fn write_text(&mut self, text: &str) -> std::io::Result<()> {
        writeln!(self.serializer.writer, "{text}")
    }

    fn write_comment(&mut self, _text: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn write_doctype(&mut self, _name: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn write_processing_instruction(&mut self, _target: &str, _data: &str) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn textify(html: &str) -> anyhow::Result<String> {
    static MANY_NEWLINES: Lazy<Regex> = lazy_regex!(r"\n{3,}");

    let doc = kuchikiki::parse_html().one(html);
    let result = serialize(&doc)?
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    let result = MANY_NEWLINES
        .replace_all(&result, "\n\n")
        .trim()
        .to_string();
    Ok(result)
}

fn serialize<T: Serialize>(node: &T) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlTextSerializer::new(&mut buf, opts);
    let opts = SerializeOpts::default();
    node.serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

#[test]
fn works() {
    let html = r"<html><body>Just a <div>test</div></body></html>";
    assert_eq!(
        textify(html).unwrap(),
        r"Just a
test"
    );
}

#[test]
fn doesnt_contain_many_newlines() {
    let html = "<html><body>bla\n\n\n\n\nblubb</body></html>";
    assert_eq!(textify(html).unwrap(), "bla\n\nblubb");
}
