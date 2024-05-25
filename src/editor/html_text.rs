use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::QualName;
use lazy_regex::regex;
use scraper::Html;

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
    let result = reserialize(html)?
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    let result = regex!(r"\n{3,}") // many newlines
        .replace_all(result.trim(), "\n\n")
        .to_string();
    Ok(result)
}

fn reserialize(html: &str) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlTextSerializer::new(&mut buf, opts);
    let opts = SerializeOpts::default();
    Html::parse_document(html).serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

#[test]
fn works() {
    let html = "<html><body>Just a <div>test</div></body></html>";
    assert_eq!(
        textify(html).unwrap(),
        "Just a
test"
    );
}

#[test]
fn doesnt_contain_many_newlines() {
    let html = "<html><body>bla\n\n\n\n\nblubb</body></html>";
    assert_eq!(textify(html).unwrap(), "bla\n\nblubb");
}
