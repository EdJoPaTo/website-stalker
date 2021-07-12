use anyhow::anyhow;
use itertools::Itertools;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Rss {
    pub title: Option<String>,
    pub item_selector: Option<String>,
    pub title_selector: Option<String>,
    pub link_selector: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_editors: Vec<Editor>,
}

impl Rss {
    /// Returns (item, title, link)
    fn parse_selectors(&self) -> anyhow::Result<(Selector, Selector, Selector)> {
        let item = if let Some(selector) = &self.item_selector {
            Selector::parse(selector)
                .map_err(|err| anyhow!("item_selector ({}) parse error: {:?}", selector, err))?
        } else {
            Selector::parse("article").unwrap()
        };
        let title = if let Some(selector) = &self.title_selector {
            Selector::parse(selector)
                .map_err(|err| anyhow!("title_selector ({}) parse error: {:?}", selector, err))?
        } else {
            Selector::parse("h2").unwrap()
        };
        let link = if let Some(selector) = &self.link_selector {
            Selector::parse(selector)
                .map_err(|err| anyhow!("link_selector ({}) parse error: {:?}", selector, err))?
        } else {
            Selector::parse("a").unwrap()
        };
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
        let (item, title, link) = self.parse_selectors()?;
        let parsed_html = scraper::Html::parse_document(html);

        let mut channel = ChannelBuilder::default();
        channel.link(url.to_string());
        channel.generator(GENERATOR.to_string());

        if let Some(title) = &self.title {
            channel.title(title.to_string());
        } else if let Some(e) = parsed_html
            .select(&Selector::parse("title").unwrap())
            .next()
        {
            channel.title(e.inner_html().trim().to_string());
        }

        if let Some(description) = parsed_html
            .select(&Selector::parse("meta[name=description]").unwrap())
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

            if let Some(link) = item.select(&link).find_map(|o| o.value().attr("href")) {
                builder.link(url.join(link)?.to_string());
            }

            let mut content = item.html();
            for editor in &self.content_editors {
                content = editor.apply(url, &content)?;
            }
            builder.content(content);

            items.push(builder.build().unwrap()); // panics on missing required
        }
        channel.items(items);

        let channel = channel.build().unwrap(); // panics on missing required
        channel.validate()?;

        let mut buffer = Vec::new();
        channel.pretty_write_to(&mut buffer, b'\t', 1)?;
        let feed = String::from_utf8(buffer)?;
        Ok(feed)
    }
}
