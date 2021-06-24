use scraper::Selector;

fn parse(selector: &str) -> anyhow::Result<Selector> {
    Selector::parse(&selector)
        .map_err(|err| anyhow::anyhow!("css selector ({}) parse error: {:?}", selector, err))
}

pub fn selector_is_valid(selector: &str) -> anyhow::Result<()> {
    parse(selector)?;
    Ok(())
}

pub fn select(html: &str, selector_str: &str) -> anyhow::Result<Vec<String>> {
    let selector = parse(selector_str)?;
    let html = scraper::Html::parse_document(html);
    let selected = html.select(&selector).map(|o| o.html()).collect::<Vec<_>>();
    Ok(selected)
}
