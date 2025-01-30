use std::sync::LazyLock;

use rss::validation::Validate;
use rss::{ChannelBuilder, ItemBuilder};
use scraper::Selector;
use serde::Deserialize;
use url::Url;

use super::Editor;

const GENERATOR: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_REPOSITORY"),);

#[derive(Debug, Clone, Deserialize)]
pub struct Rss {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_selector_opt"
    )]
    pub item_selector: Option<Selector>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_selector_opt"
    )]
    pub title_selector: Option<Selector>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_selector_opt"
    )]
    pub link_selector: Option<Selector>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_editors: Vec<Editor>,
}

macro_rules! selector {
    ($selector:literal) => {{
        static SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse($selector).unwrap());
        &SELECTOR
    }};
}

impl Rss {
    fn item_selector(&self) -> &Selector {
        static FALLBACK: LazyLock<Selector> = LazyLock::new(|| Selector::parse("article").unwrap());
        self.item_selector.as_ref().unwrap_or_else(|| &FALLBACK)
    }

    fn title_selector(&self) -> &Selector {
        static FALLBACK: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h2").unwrap());
        self.title_selector.as_ref().unwrap_or_else(|| &FALLBACK)
    }

    fn link_selector(&self) -> &Selector {
        static FALLBACK: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a").unwrap());
        self.link_selector.as_ref().unwrap_or_else(|| &FALLBACK)
    }

    pub fn generate(&self, url: &Url, html: &str) -> anyhow::Result<String> {
        let html = scraper::Html::parse_document(html);

        let mut channel = ChannelBuilder::default();
        channel.link(url.to_string());
        channel.generator(GENERATOR.to_owned());

        if let Some(title) = &self.title {
            channel.title(title.to_string());
        } else if let Some(element) = html
            .select(selector!("title, h1, h2, h3, h4, h5, h6"))
            .next()
        {
            channel.title(element.inner_html().trim().to_owned());
        } else {
            crate::logger::warn(&format!(
                "RSS Feed has no title from html or the config: {url}"
            ));
        }

        if let Some(description) = html
            .select(selector!("meta[name=description]"))
            .find_map(|element| element.value().attr("content"))
        {
            channel.description(description.to_owned());
        }

        let mut items = Vec::new();
        for item in html.select(self.item_selector()) {
            let mut builder = ItemBuilder::default();

            if let Some(title) = item.select(self.title_selector()).next() {
                builder.title(
                    title
                        .text()
                        .map(str::trim)
                        .filter(|title| !title.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }

            // When the item is the link itself
            if let Some(link) = item.value().attr("href") {
                builder.link(url.join(link)?.to_string());
            }

            if let Some(link) = item
                .select(self.link_selector())
                .find_map(|element| element.value().attr("href"))
            {
                builder.link(url.join(link)?.to_string());
            }

            if let Some(bla) = item
                .select(selector!("*[datetime]"))
                .find_map(|element| element.value().attr("datetime"))
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            {
                builder.pub_date(bla.to_rfc2822());
            }

            let content = super::Content {
                extension: Some("html"),
                text: item.html(),
            };
            let content = Editor::apply_many(&self.content_editors, url, content)?;
            builder.content(content.text);

            items.push(builder.build());
        }
        anyhow::ensure!(!items.is_empty(), "rss item_selector selected nothing");
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
    assert!(result.contains("website-stalker"));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains("<link>https://edjopato.de/posts/a/</link>"));
    assert!(result.contains("<link>https://edjopato.de/posts/b/</link>"));
    assert!(result.contains("<title>Whatever</title>"));
    assert!(result.contains("<title>First</title>"));
    assert!(result.contains("<title>Second</title>"));
    assert!(!result.contains("ignore"));
    Ok(())
}

#[test]
#[should_panic = "item_selector selected nothing"]
fn example_with_no_items_errors() {
    let html = "<html>
	<head>
        <title>Whatever</title>
	</head>
	<body>
		ignore
	</body>
</html>";
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
        item_selector: Some(Selector::parse("a").unwrap()),
        title_selector: None,
        link_selector: None,
        content_editors: vec![],
    };
    let url = &Url::parse("https://edjopato.de/posts/").unwrap();
    let result = rss.generate(url, html).unwrap();
    println!("{result}");
    assert!(result.contains("website-stalker"));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains("<link>https://edjopato.de/posts/a/</link>"));
    assert!(result.contains("<link>https://edjopato.de/posts/b/</link>"));
    assert!(result.contains("<title>Whatever</title>"));
    assert!(result.contains("<title>First</title>"));
    assert!(result.contains("<title>Second</title>"));
    assert!(!result.contains("ignore"));
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
        title: Some("My title".to_owned()),
        item_selector: Some(Selector::parse(".entry").unwrap()),
        title_selector: Some(Selector::parse("h6").unwrap()),
        link_selector: Some(Selector::parse("a:last-of-type").unwrap()),
        content_editors: vec![Editor::HtmlTextify],
    };

    let url = &Url::parse("https://edjopato.de/posts/").unwrap();
    let result = rss.generate(url, html).unwrap();
    println!("{result}");
    assert!(result.contains("website-stalker"));
    assert!(result.contains(r#"<rss version="2.0" "#));
    assert!(result.contains("<link>https://edjopato.de/posts/a/</link>"));
    assert!(result.contains("<link>https://edjopato.de/posts/b/</link>"));
    assert!(result.contains("<title>My title</title>"));
    assert!(result.contains("<title>First</title>"));
    assert!(result.contains("<title>Second</title>"));
    assert!(!result.contains("buy-now"));
    assert!(!result.contains("Whatever"));
}
