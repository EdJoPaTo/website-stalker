use lazy_regex::{lazy_regex, Captures, Lazy, Regex};

pub fn markdownify(html: &str) -> String {
    static LINK: Lazy<Regex> = lazy_regex!(r"\[([^\]]+)\]\(([^)]+)\)");
    static MANY_SPACES: Lazy<Regex> = lazy_regex!(r"\s+");

    let result = html2md::parse_html(html)
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");

    let result = LINK.replace_all(&result, |cap: &Captures| {
        let label = MANY_SPACES.replace_all(cap[1].trim(), " ");
        let url = cap[2].trim();
        if label == url {
            format!("<{url}>")
        } else {
            format!("[{label}]({url})")
        }
    });

    result.to_string()
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
