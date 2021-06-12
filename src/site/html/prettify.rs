use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::tendril::TendrilSink;
use html5ever::QualName;

struct HtmlPrettySerializer<Wr: Write> {
    pub serializer: HtmlSerializer<Wr>,
    depth: usize,
}

impl<Wr: Write> HtmlPrettySerializer<Wr> {
    pub fn new(writer: Wr, opts: SerializeOpts) -> Self {
        Self {
            serializer: HtmlSerializer::new(writer, opts),
            depth: 0,
        }
    }

    fn indent(&mut self) -> std::io::Result<()> {
        for _ in 0..self.depth {
            self.serializer.writer.write_all(b"\t")?;
        }
        Ok(())
    }
}

impl<Wr: Write> Serializer for HtmlPrettySerializer<Wr> {
    fn start_elem<'a, AttrIter>(&mut self, name: QualName, attrs: AttrIter) -> std::io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        self.indent()?;
        self.depth += 1;

        self.serializer.start_elem(name, attrs)?;
        self.serializer.writer.write_all(b"\n")
    }

    fn end_elem(&mut self, name: QualName) -> std::io::Result<()> {
        self.depth = self.depth.saturating_sub(1);
        self.indent()?;

        self.serializer.end_elem(name)?;
        self.serializer.writer.write_all(b"\n")
    }

    fn write_text(&mut self, text: &str) -> std::io::Result<()> {
        self.indent()?;
        self.serializer.write_text(text)?;
        self.serializer.writer.write_all(b"\n")
    }

    fn write_comment(&mut self, text: &str) -> std::io::Result<()> {
        self.indent()?;
        self.serializer.write_comment(text)?;
        self.serializer.writer.write_all(b"\n")
    }

    fn write_doctype(&mut self, name: &str) -> std::io::Result<()> {
        self.indent()?;
        self.serializer.write_doctype(name)?;
        self.serializer.writer.write_all(b"\n")
    }

    fn write_processing_instruction(&mut self, target: &str, data: &str) -> std::io::Result<()> {
        self.indent()?;
        self.serializer.write_processing_instruction(target, data)?;
        self.serializer.writer.write_all(b"\n")
    }
}

pub fn prettify(html: &str) -> anyhow::Result<String> {
    let doc = kuchiki::parse_html().one(html);
    let result = serialize(&doc)?
        .lines()
        .map(str::trim_end)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(result)
}

fn serialize<T: Serialize>(node: &T) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlPrettySerializer::new(&mut buf, opts);
    let opts = SerializeOpts::default();
    node.serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

#[test]
fn prettify_works() {
    let ugly = r#"<html><body>Just a <div>test</div></body></html>"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        r#"<html>
	<head>
	</head>
	<body>
		Just a
		<div>
			test
		</div>
	</body>
</html>"#
    );
}
