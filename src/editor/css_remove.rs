use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CssRemover(String);

impl CssRemover {
    fn parse(&self) -> anyhow::Result<scraper::Selector> {
        let scrape_selector = scraper::Selector::parse(&self.0)
            .map_err(|err| anyhow::anyhow!("css remover ({}) parse error: {:?}", self.0, err))?;

        Ok(scrape_selector)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse()?;
        Ok(())
    }

    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let parsed_html = scraper::Html::parse_document(html);
        let selected = parsed_html
            .select(&self.parse()?)
            .map(|o| o.html())
            .collect::<Vec<_>>();

        let mut html = parsed_html.root_element().html();
        for s in selected {
            html = html.replace(&s, "");
        }
        Ok(html)
    }
}

impl std::str::FromStr for CssRemover {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = Self(s.to_string());
        s.parse()?;
        Ok(s)
    }
}

#[test]
fn valid() {
    let s = CssRemover("body".to_string());
    let result = s.is_valid();
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
#[should_panic = "parse error"]
fn invalid() {
    CssRemover(".".to_string()).is_valid().unwrap();
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn removes_tag() {
    let remover = CssRemover("p".to_string());
    let html = remover.apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b">B</div></body></html>"#
    );
}

#[test]
fn remove_not_found() {
    let remover = CssRemover("p".to_string());
    let html = remover
        .apply(r#"<html><head></head><body>test</body></html>"#)
        .unwrap();
    assert_eq!(html, r#"<html><head></head><body>test</body></html>"#);
}
