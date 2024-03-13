#![allow(deprecated)] // TODO: remove on breaking release

use once_cell::sync::Lazy;
use url::Url;

static DEFAULT_MUSTACHE_TEMPLATE: Lazy<mustache::Template> = Lazy::new(|| {
    mustache::compile_str(
        "
{{#singlehost}}
{{.}} changed
{{/singlehost}}
{{^singlehost}}
{{siteamount}} websites changed
{{/singlehost}}

{{#sites}}
- {{.}}
{{/sites}}

{{#commit}}
See {{.}}
{{/commit}}
",
    )
    .unwrap()
});

#[derive(serde::Serialize)]
pub struct MustacheData {
    commit: Option<String>,
    #[deprecated = "use singlehost"]
    singledomain: Option<String>,
    singlehost: Option<String>,
    siteamount: usize,

    #[deprecated = "use hosts"]
    domains: Vec<String>,
    hosts: Vec<String>,
    sites: Vec<Url>,
}

impl MustacheData {
    pub fn new(commit: Option<String>, changed_urls: Vec<Url>) -> Self {
        let mut sites = changed_urls;
        sites.sort_unstable();
        sites.dedup();

        let mut hosts = sites
            .iter()
            .filter_map(Url::host_str)
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        hosts.dedup();

        let singlehost = if let [single] = hosts.as_slice() {
            Some(single.clone())
        } else {
            None
        };

        Self {
            commit,
            singledomain: singlehost.clone(),
            singlehost,
            siteamount: sites.len(),

            domains: hosts.clone(),
            hosts,
            sites,
        }
    }

    pub fn apply_to_template(
        &self,
        template: Option<&mustache::Template>,
    ) -> anyhow::Result<String> {
        let template = template.unwrap_or_else(|| &DEFAULT_MUSTACHE_TEMPLATE);
        Ok(template.render_to_string(self)?.trim().to_owned())
    }

    fn example_single(commit: Option<&str>) -> Self {
        Self::new(
            commit.map(ToOwned::to_owned),
            vec![Url::parse("https://edjopato.de/post/").unwrap()],
        )
    }

    fn example_different(commit: Option<&str>) -> Self {
        Self::new(
            commit.map(ToOwned::to_owned),
            vec![
                Url::parse("https://edjopato.de/post/").unwrap(),
                Url::parse("https://foo.bar/").unwrap(),
            ],
        )
    }

    fn example_same(commit: Option<&str>) -> Self {
        Self::new(
            commit.map(ToOwned::to_owned),
            vec![
                Url::parse("https://edjopato.de/").unwrap(),
                Url::parse("https://edjopato.de/post/").unwrap(),
            ],
        )
    }
}

pub fn validate_template(template: &mustache::Template) -> anyhow::Result<()> {
    const DEPRECATED_TEXT: &str = "do not use deprecated field";

    let singledomain = template
        .render_to_string(&MustacheData {
            commit: None,
            singledomain: Some(DEPRECATED_TEXT.to_owned()),
            singlehost: None,
            siteamount: 42,
            domains: vec![],
            hosts: vec![],
            sites: vec![],
        })?
        .contains(DEPRECATED_TEXT);
    if singledomain {
        crate::logger::warn(
            "Replace singledomain with singlehost in template. singledomain will be removed in the future.",
        );
    }

    let domains = template
        .render_to_string(&MustacheData {
            commit: None,
            singledomain: None,
            singlehost: None,
            siteamount: 42,
            domains: vec![DEPRECATED_TEXT.to_owned()],
            hosts: vec![],
            sites: vec![],
        })?
        .contains(DEPRECATED_TEXT);
    if domains {
        crate::logger::warn(
            "Replace domains with hosts in template. domains will be removed in the future.",
        );
    }

    let template = Some(template);
    let any_empty = [
        MustacheData::example_single(Some("666")).apply_to_template(template)?,
        MustacheData::example_single(None).apply_to_template(template)?,
        MustacheData::example_different(Some("666")).apply_to_template(template)?,
        MustacheData::example_different(None).apply_to_template(template)?,
        MustacheData::example_same(Some("666")).apply_to_template(template)?,
        MustacheData::example_same(None).apply_to_template(template)?,
    ]
    .iter()
    .any(std::string::String::is_empty);

    if any_empty {
        Err(anyhow::anyhow!("template produced empty notification text"))
    } else {
        Ok(())
    }
}

#[test]
fn simple_template_is_valid() {
    let template = mustache::compile_str("Hello {{name}}").unwrap();
    validate_template(&template).unwrap();
}

#[test]
fn default_template_is_valid() {
    validate_template(&DEFAULT_MUSTACHE_TEMPLATE).unwrap();
}

#[test]
fn notification_message_for_two_same_domain_sites() {
    let text = MustacheData::example_same(Some("1234abc"))
        .apply_to_template(None)
        .unwrap();
    assert_eq!(
        text,
        "edjopato.de changed

- https://edjopato.de/
- https://edjopato.de/post/

See 1234abc"
    );
}

#[test]
fn notification_message_for_two_different_domain_sites() {
    let text = MustacheData::example_different(Some("1234abc"))
        .apply_to_template(None)
        .unwrap();
    assert_eq!(
        text,
        "2 websites changed

- https://edjopato.de/post/
- https://foo.bar/

See 1234abc"
    );
}

#[test]
fn notification_message_for_single_site_without_commit() {
    let text = MustacheData::example_single(None)
        .apply_to_template(None)
        .unwrap();
    assert_eq!(
        text,
        "edjopato.de changed

- https://edjopato.de/post/"
    );
}
