use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::QualName;
use scraper::Html;

struct HtmlPrettySerializer<Wr: Write> {
    serializer: HtmlSerializer<Wr>,
    depth: usize,
}

impl<Wr: Write> HtmlPrettySerializer<Wr> {
    fn new(writer: Wr, opts: SerializeOpts) -> Self {
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

        let mut attrs = attrs
            .filter_map(|(name, value)| match name.local.as_ref() {
                "class" => {
                    let mut classes = value.split_whitespace().collect::<Vec<_>>();
                    if classes.is_empty() {
                        None
                    } else {
                        classes.sort_unstable();
                        Some((name, classes.join(" ")))
                    }
                }
                "style" => {
                    let mut statements = value
                        .split(';')
                        .map(str::trim)
                        .filter(|statement| !statement.is_empty())
                        .map(format_css_statement)
                        .collect::<Vec<_>>();
                    if statements.is_empty() {
                        None
                    } else {
                        statements.sort_unstable();
                        Some((name, statements.join(" ")))
                    }
                }
                _ => Some((name, value.to_owned())),
            })
            .collect::<Vec<_>>();
        attrs.sort();
        let attrs = attrs.iter().map(|(name, value)| (*name, value.as_str()));

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
        let text = text.trim();
        if text.is_empty() {
            Ok(())
        } else {
            self.indent()?;
            writeln!(self.serializer.writer, "{}", text.trim())
        }
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
    let result = reserialize(html)?
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(result)
}

fn reserialize(html: &str) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlPrettySerializer::new(&mut buf, opts);
    let opts = SerializeOpts::default();
    Html::parse_document(html).serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

/// Never receives the ; splitting them
fn format_css_statement(content: &str) -> String {
    let content = content.trim();
    if let Some((key, value)) = content.split_once(':') {
        format!("{}: {};", key.trim_end(), value.trim_start())
    } else {
        format!("{content};")
    }
}

#[test]
fn format_css_statement_works() {
    assert_eq!("color: white;", format_css_statement("color:  white"));
    assert_eq!("color: white;", format_css_statement("color:white"));
    assert_eq!("color: white;", format_css_statement("  color: white  "));
}

#[test]
fn works() {
    let ugly = "<html><body>Just a <div>test</div></body></html>";
    assert_eq!(
        prettify(ugly).unwrap(),
        "<html>
	<head>
	</head>
	<body>
		Just a
		<div>
			test
		</div>
	</body>
</html>"
    );
}

#[test]
fn attributes_sorted() {
    let ugly = r#"<html><body><div style="--a: 42;" class="a">test</div></body></html>"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        r#"<html>
	<head>
	</head>
	<body>
		<div class="a" style="--a: 42;">
			test
		</div>
	</body>
</html>"#
    );
}

#[test]
fn classes_sorted() {
    let ugly = r#"<html><body><div class="b  a">test</div></body></html>"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        r#"<html>
	<head>
	</head>
	<body>
		<div class="a b">
			test
		</div>
	</body>
</html>"#
    );
}

#[test]
fn remove_empty_class_attribute() {
    let ugly = r#"<html><body><div class=" ">test</div></body></html>"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        "<html>
	<head>
	</head>
	<body>
		<div>
			test
		</div>
	</body>
</html>"
    );
}

#[test]
fn style_formatted() {
    let ugly = r#"<html><body><div style="display:none;color:white;--something:42;">test</div></body></html>"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        r#"<html>
	<head>
	</head>
	<body>
		<div style="--something: 42; color: white; display: none;">
			test
		</div>
	</body>
</html>"#
    );
}
