use lazy_regex::{regex, Captures};

pub fn markdownify(html: &str) -> String {
    let markdown = html2md::parse_html(html)
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");

    // Simplify Markdown links
    let markdown = regex!(r"\[([^\]]+)\]\(([^)]+)\)").replace_all(&markdown, |cap: &Captures| {
        // prevent many spaces on the label
        let label = regex!(r"\s+").replace_all(cap[1].trim(), " ");
        let url = cap[2].trim();
        if label == url {
            format!("<{url}>")
        } else {
            format!("[{label}]({url})")
        }
    });

    markdown.to_string()
}

#[cfg(test)]
#[track_caller]
fn case(html: &str, expected: &str) {
    let result = dbg!(markdownify(html));
    let raw = dbg!(html2md::parse_html(html));
    assert_eq!(result, expected);
    assert_ne!(result, raw, "special handling no longer needed");
}

#[test]
fn angled_url() {
    let html = r#"<a href="https://edjopato.de/">https://edjopato.de/</a>"#;
    case(html, "<https://edjopato.de/>");
}

#[test]
fn link_label_trim_simple() {
    let html = r#"<a href="/"> bla </a>"#;
    case(html, "[bla](/)");
}

#[test]
fn link_label_trim_multiline() {
    let html = r#"<a href="/"><div>bla</div><div>blubb</div></a>"#;
    case(html, "[bla blubb](/)");
}

#[test]
fn trim_lineendings() {
    // \u{a0} is NO-BREAK SPACE
    let html = "<p>whatever  <br>\nis\t<br>\nthis \u{a0}<br>\nmeh</p>";
    case(html, "whatever\nis\nthis\nmeh");
}
