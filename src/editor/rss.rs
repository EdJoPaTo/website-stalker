use anyhow::anyhow;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rss::validation::Validate;
use rss::{ChannelBuilder, ItemBuilder};
use scraper::Selector;
use serde::{Deserialize, Serialize};
use url::Url;

use super::Editor;

const GENERATOR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rss {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_selector: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_editors: Vec<Editor>,
}

impl Rss {
    fn item_selector(&self) -> &str {
        self.item_selector.as_deref().unwrap_or("article")
    }
    fn title_selector(&self) -> &str {
        self.title_selector.as_deref().unwrap_or("h2")
    }
    fn link_selector(&self) -> &str {
        self.link_selector.as_deref().unwrap_or("a")
    }

    /// Returns (item, title, link)
    fn parse_selectors(&self) -> anyhow::Result<(Selector, Selector, Selector)> {
        let item = self.item_selector();
        let item = Selector::parse(item)
            .map_err(|err| anyhow!("item_selector ({item}) parse error: {err:?}"))?;

        let title = self.title_selector();
        let title = Selector::parse(title)
            .map_err(|err| anyhow!("title_selector ({title}) parse error: {err:?}"))?;

        let link = self.link_selector();
        let link = Selector::parse(link)
            .map_err(|err| anyhow!("link_selector ({link}) parse error: {err:?}"))?;

        Ok((item, title, link))
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse_selectors()?;
        for editor in &self.content_editors {
            editor.is_valid()?;
        }
        Ok(())
    }

    pub fn generate(&self, url: &Url, html: &str) -> anyhow::Result<String> {
        static TITLE: Lazy<Selector> = Lazy::new(|| Selector::parse("title").unwrap());
        static DESCRIPTION: Lazy<Selector> =
            Lazy::new(|| Selector::parse("meta[name=description]").unwrap());
        static DATETIME: Lazy<Selector> = Lazy::new(|| Selector::parse("*[datetime]").unwrap());

        let (item, title, link) = self.parse_selectors()?;
        let parsed_html = scraper::Html::parse_document(html);

        let mut channel = ChannelBuilder::default();
        channel.link(url.to_string());
        channel.generator(GENERATOR.to_string());

        if let Some(title) = &self.title {
            channel.title(title.to_string());
        } else if let Some(e) = parsed_html.select(&TITLE).next() {
            channel.title(e.inner_html().trim().to_string());
        }

        if let Some(description) = parsed_html
            .select(&DESCRIPTION)
            .find_map(|e| e.value().attr("content"))
        {
            channel.description(description.to_string());
        }

        let mut items = Vec::new();
        for item in parsed_html.select(&item) {
            let mut builder = ItemBuilder::default();

            if let Some(title) = item.select(&title).next() {
                builder.title(title.text().map(str::trim).join("\n").trim().to_string());
            }

            // When the item is the link itself
            if let Some(link) = item.value().attr("href") {
                builder.link(url.join(link)?.to_string());
            }

            if let Some(link) = item.select(&link).find_map(|o| o.value().attr("href")) {
                builder.link(url.join(link)?.to_string());
            }

            if let Some(bla) = item
                .select(&DATETIME)
                .find_map(|o| o.value().attr("datetime"))
                .and_then(|o| chrono::DateTime::parse_from_rfc3339(o).ok())
            {
                builder.pub_date(bla.to_rfc2822());
            }

            let mut content = super::Content {
                extension: Some("html"),
                text: item.html(),
            };
            for editor in &self.content_editors {
                content = editor.apply(url, &content)?;
            }
            builder.content(content.text);

            items.push(builder.build());
        }
        if items.is_empty() {
            anyhow::bail!(
                "rss item_selector ({}) selected nothing",
                self.item_selector()
            );
        }
        channel.items(items);

        let channel = channel.build();
        channel.validate()?;

        let mut buffer = Vec::new();
        channel.pretty_write_to(&mut buffer, b'\t', 1)?;
        let feed = String::from_utf8(buffer)?;
        Ok(feed)
    }
}

#[test]
fn minimal_options_are_valid() {
    let rss = Rss {
        title: None,
        item_selector: None,
        title_selector: None,
        link_selector: None,
        content_editors: vec![],
    };
    let result = rss.is_valid();
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn example_with_defaults_works() -> anyhow::Result<()> {
    let html = r#"<html>
	<head>
        <title>Whatever</title>
	</head>
	<body>
		ignore
		<article>
			<h2>First</h2>
            <a href="a/">Link</a>
            content
		</article>
		<article>
			<h2>Second</h2>
            <a href="b/">Link</a>
            lorem
		</article>
	</body>
</html>"#;
    let rss = Rss {
        title: None,
        item_selector: None,
        title_selector: None,
        link_selector: None,
        content_editors: vec![],
    };
    let result = rss.generate(&Url::parse("https://edjopato.de/posts/")?, html)?;
    println!("{result}");
    assert!(result.contains(r#"website-stalker"#));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/a/</link>"#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/b/</link>"#));
    assert!(result.contains(r#"<title>Whatever</title>"#));
    assert!(result.contains(r#"<title>First</title>"#));
    assert!(result.contains(r#"<title>Second</title>"#));
    assert!(!result.contains(r#"ignore"#));
    Ok(())
}

#[test]
#[should_panic = "item_selector (article) selected nothing"]
fn example_with_no_items_errors() {
    let html = r#"<html>
	<head>
        <title>Whatever</title>
	</head>
	<body>
		ignore
	</body>
</html>"#;
    let rss = Rss {
        title: None,
        item_selector: None,
        title_selector: None,
        link_selector: None,
        content_editors: vec![],
    };
    let url = Url::parse("https://edjopato.de/posts/").unwrap();
    rss.generate(&url, html).unwrap();
}

#[test]
fn example_with_item_equals_link() {
    let html = r#"<html>
	<head>
        <title>Whatever</title>
	</head>
	<body>
		ignore
		<article>
        <a href="a/">
			<h2>First</h2>
            content
		</a>
		<a href="b/">
			<h2>Second</h2>
            lorem
		</a>
	</body>
</html>"#;
    let rss = Rss {
        title: None,
        item_selector: Some("a".to_string()),
        title_selector: None,
        link_selector: None,
        content_editors: vec![],
    };
    let url = &Url::parse("https://edjopato.de/posts/").unwrap();
    let result = rss.generate(url, html).unwrap();
    println!("{result}");
    assert!(result.contains(r#"website-stalker"#));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/a/</link>"#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/b/</link>"#));
    assert!(result.contains(r#"<title>Whatever</title>"#));
    assert!(result.contains(r#"<title>First</title>"#));
    assert!(result.contains(r#"<title>Second</title>"#));
    assert!(!result.contains(r#"ignore"#));
}

#[test]
fn ugly_example_works() {
    let html = r#"<html>
	<head>
        <title>Whatever</title>
	</head>
	<body>
		<div class="entry">
			<h6>First</h6>
            <a href="/buy-now/">Ad</a>
            <a href="a/">Link</a>
            content
		</div>
		<div class="entry">
			<h6>Second</h6>
            <a href="/buy-now/">Ad</a>
            <a href="b/">Link</a>
            lorem
		</div>
	</body>
</html>"#;
    let rss = Rss {
        title: Some("My title".to_string()),
        item_selector: Some(".entry".to_string()),
        title_selector: Some("h6".to_string()),
        link_selector: Some("a:last-of-type".to_string()),
        content_editors: vec![Editor::HtmlTextify],
    };
    let valid = rss.is_valid();
    println!("is_valid {valid:?}");
    assert!(valid.is_ok(), "is_valid");

    let url = &Url::parse("https://edjopato.de/posts/").unwrap();
    let result = rss.generate(url, html).unwrap();
    println!("{result}");
    assert!(result.contains(r#"website-stalker"#));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/a/</link>"#));
    assert!(result.contains(r#"<link>https://edjopato.de/posts/b/</link>"#));
    assert!(result.contains(r#"<title>My title</title>"#));
    assert!(result.contains(r#"<title>First</title>"#));
    assert!(result.contains(r#"<title>Second</title>"#));
    assert!(!result.contains(r#"buy-now"#));
    assert!(!result.contains(r#"Whatever"#));
}
