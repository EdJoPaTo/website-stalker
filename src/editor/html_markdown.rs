use std::io::Write;

use html5ever::serialize::{AttrRef, HtmlSerializer, Serialize, SerializeOpts, Serializer};
use html5ever::tendril::TendrilSink;
use html5ever::QualName;
use regex::Regex;

struct HtmlMarkdownSerializer<Wr: Write> {
    serializer: HtmlSerializer<Wr>,
    bold: bool,
    italic: bool,
    a_href: Option<String>,
    ul_depth: usize,
}

impl<Wr: Write> HtmlMarkdownSerializer<Wr> {
    fn new(writer: Wr, opts: SerializeOpts) -> Self {
        Self {
            serializer: HtmlSerializer::new(writer, opts),
            bold: false,
            italic: false,
            a_href: None,
            ul_depth: 0,
        }
    }
}

fn get_attr_value<'a, AttrIter>(attrs: &mut AttrIter, name: &str) -> Option<&'a str>
where
    AttrIter: Iterator<Item = AttrRef<'a>>,
{
    attrs.find_map(|(q, value)| {
        if q.local.to_string() == name {
            Some(value)
        } else {
            None
        }
    })
}

impl<Wr: Write> Serializer for HtmlMarkdownSerializer<Wr> {
    fn start_elem<'a, AttrIter>(
        &mut self,
        name: QualName,
        mut attrs: AttrIter,
    ) -> std::io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        let local = name.local.to_string();
        match local.as_ref() {
            "h1" => {
                self.serializer.writer.write_all(b"\n# ")?;
            }
            "h2" => {
                self.serializer.writer.write_all(b"\n## ")?;
            }
            "h3" => {
                self.serializer.writer.write_all(b"\n### ")?;
            }
            "h4" => {
                self.serializer.writer.write_all(b"\n#### ")?;
            }
            "h5" => {
                self.serializer.writer.write_all(b"\n##### ")?;
            }
            "h6" => {
                self.serializer.writer.write_all(b"\n###### ")?;
            }
            "p" | "br" => {
                self.serializer.writer.write_all(b"\n")?;
            }
            "ul" => {
                self.ul_depth = self.ul_depth.saturating_add(1);
            }
            "li" => {
                if self.ul_depth > 0 {
                    let amount = self.ul_depth - 1;
                    let indent = "	".repeat(amount);
                    self.serializer.write_text(&indent)?;
                    self.serializer.writer.write_all(b"- ")?;
                }
            }
            "b" | "strong" => {
                self.bold = true;
            }
            "i" | "em" => {
                self.italic = true;
            }
            "a" => {
                self.a_href = get_attr_value(&mut attrs, "href")
                    .filter(|o| !o.is_empty())
                    .map(std::string::ToString::to_string);
            }
            "img" => {
                let alt = get_attr_value(&mut attrs, "alt").unwrap_or_default();
                let src = get_attr_value(&mut attrs, "src").filter(|o| !o.is_empty());
                if let Some(src) = src {
                    let line = format!("![{}]({})\n", alt, src);
                    self.serializer.write_text(&line)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn end_elem(&mut self, name: QualName) -> std::io::Result<()> {
        let local = name.local.to_string();
        match local.as_ref() {
            "p" => {
                self.serializer.writer.write_all(b"\n")?;
            }
            "ul" => {
                self.ul_depth = self.ul_depth.saturating_sub(1);
            }
            "b" | "strong" => {
                self.bold = false;
            }
            "i" | "em" => {
                self.italic = false;
            }
            "a" => {
                self.a_href = None;
            }
            _ => {}
        }

        Ok(())
    }

    fn write_text(&mut self, text: &str) -> std::io::Result<()> {
        let text = text.trim();
        if !text.is_empty() {
            if self.bold {
                self.serializer.writer.write_all(b"**")?;
            }
            if self.italic {
                self.serializer.writer.write_all(b"*")?;
            }
            if self.a_href.is_some() {
                self.serializer.writer.write_all(b"[")?;
            }

            self.serializer.write_text(text)?;

            if let Some(href) = &self.a_href {
                self.serializer.writer.write_all(b"](")?;
                self.serializer.write_text(href)?;
                self.serializer.writer.write_all(b")")?;
            }
            if self.italic {
                self.serializer.writer.write_all(b"*")?;
            }
            if self.bold {
                self.serializer.writer.write_all(b"**")?;
            }

            self.serializer.writer.write_all(b"\n")?;
        }
        Ok(())
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

pub fn markdownify(html: &str) -> anyhow::Result<String> {
    let doc = kuchiki::parse_html().one(html);
    let result = serialize(&doc)?
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    let result = Regex::new("\n{3,}")
        .unwrap()
        .replace_all(&result, "\n\n")
        .trim_matches('\n')
        .to_string();
    Ok(result)
}

fn serialize<T: Serialize>(node: &T) -> anyhow::Result<String> {
    let mut buf = Vec::new();

    let opts = SerializeOpts::default();
    let mut ser = HtmlMarkdownSerializer::new(&mut buf, opts);
    let opts = SerializeOpts::default();
    node.serialize(&mut ser, opts.traversal_scope)?;

    let result = String::from_utf8(buf)?;
    Ok(result)
}

#[test]
fn example_works() {
    let html = r#"<html><body>Just<h1>some</h1>random<strong>stuff</strong>with<h3>some</h3>content<div>test</div></body></html>"#;
    assert_eq!(
        markdownify(html).unwrap(),
        r#"Just

# some
random
**stuff**
with

### some
content
test"#
    );
}

#[test]
fn link_works() {
    let html = r#"<html><body>Just<a href="https://edjopato.de/">some</a>test</body></html>"#;
    assert_eq!(
        markdownify(html).unwrap(),
        r#"Just
[some](https://edjopato.de/)
test"#
    );
}

#[test]
fn img_works() {
    let html =
        r#"<html><body>Just<img src="some.jpg" title="Titel" alt="Alternative">test</body></html>"#;
    assert_eq!(
        markdownify(html).unwrap(),
        r#"Just
![Alternative](some.jpg)
test"#
    );
}

#[test]
fn list_works() {
    // See https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul
    let html = r#"<html><body>Just
<ul>
    <li>first item</li>
    <li>second item
    <!-- Look, the closing </li> tag is not placed here! -->
        <ul>
            <li>second item first subitem</li>
            <li>second item second subitem
            <!-- Same for the second nested unordered list! -->
                <ul>
                    <li>second item second subitem first sub-subitem</li>
                    <li>second item second subitem second sub-subitem</li>
                    <li>second item second subitem third sub-subitem</li>
                </ul>
            </li> <!-- Closing </li> tag for the li that contains the third unordered list -->
            <li>second item third subitem</li>
        </ul>
    <!-- Here is the closing </li> tag -->
    </li>
    <li>third item</li>
</ul>
test</body></html>"#;
    assert_eq!(
        markdownify(html).unwrap(),
        r#"Just
- first item
- second item
	- second item first subitem
	- second item second subitem
		- second item second subitem first sub-subitem
		- second item second subitem second sub-subitem
		- second item second subitem third sub-subitem
	- second item third subitem
- third item
test"#
    );
}
