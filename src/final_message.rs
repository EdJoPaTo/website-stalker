use std::fmt::Write;

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
pub struct FinalMessage {
    #[deprecated = "use hosts"]
    domains: Vec<String>,
    hosts: Vec<String>,
    sites: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct MustacheData {
    commit: Option<String>,
    #[deprecated = "use singlehost"]
    singledomain: Option<String>,
    singlehost: Option<String>,
    siteamount: usize,

    #[serde(flatten)]
    msg: FinalMessage,
}

impl FinalMessage {
    pub fn new(changed_urls: &[Url]) -> Self {
        let mut hosts = changed_urls
            .iter()
            .filter_map(Url::host_str)
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        hosts.sort_unstable();
        hosts.dedup();

        let mut sites = changed_urls
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        sites.sort();
        sites.dedup();

        Self {
            domains: hosts.clone(),
            hosts,
            sites,
        }
    }

    pub fn to_commit(&self) -> String {
        let mut text = match self.hosts.as_slice() {
            [] => "just background magic 🧽🔮🧹\n\ncleanup or updating meta files".to_owned(),
            [single] => format!("🌐👀 {single}\n\n"),
            _ => format!("🌐👀 stalked {} website changes\n\n", self.sites.len()),
        };
        for site in &self.sites {
            _ = writeln!(&mut text, "- {site}");
        }
        text
    }

    pub fn into_mustache_data(self, commit: Option<String>) -> MustacheData {
        let singlehost = if let [single] = self.hosts.as_slice() {
            Some(single.clone())
        } else {
            None
        };

        MustacheData {
            commit,
            singledomain: singlehost.clone(),
            singlehost,
            siteamount: self.sites.len(),
            msg: self,
        }
    }

    fn example_single() -> Self {
        Self::new(&[Url::parse("https://edjopato.de/post/").unwrap()])
    }

    fn example_different() -> Self {
        Self::new(&[
            Url::parse("https://edjopato.de/post/").unwrap(),
            Url::parse("https://foo.bar/").unwrap(),
        ])
    }

    fn example_same() -> Self {
        Self::new(&[
            Url::parse("https://edjopato.de/").unwrap(),
            Url::parse("https://edjopato.de/post/").unwrap(),
        ])
    }
}

impl MustacheData {
    pub fn apply_to_template(
        &self,
        template: Option<&mustache::Template>,
    ) -> anyhow::Result<String> {
        let template = template.unwrap_or_else(|| &DEFAULT_MUSTACHE_TEMPLATE);
        Ok(template.render_to_string(self)?.trim().to_owned())
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
            msg: FinalMessage {
                domains: vec![],
                hosts: vec![],
                sites: vec![],
            },
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
            msg: FinalMessage {
                domains: vec![DEPRECATED_TEXT.to_owned()],
                hosts: vec![],
                sites: vec![],
            },
        })?
        .contains(DEPRECATED_TEXT);
    if domains {
        crate::logger::warn(
            "Replace domains with hosts in template. domains will be removed in the future.",
        );
    }

    let template = Some(template);
    let any_empty = [
        FinalMessage::example_single()
            .into_mustache_data(Some("666".into()))
            .apply_to_template(template)?,
        FinalMessage::example_single()
            .into_mustache_data(None)
            .apply_to_template(template)?,
        FinalMessage::example_different()
            .into_mustache_data(Some("666".into()))
            .apply_to_template(template)?,
        FinalMessage::example_different()
            .into_mustache_data(None)
            .apply_to_template(template)?,
        FinalMessage::example_same()
            .into_mustache_data(Some("666".into()))
            .apply_to_template(template)?,
        FinalMessage::example_same()
            .into_mustache_data(None)
            .apply_to_template(template)?,
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
fn commit_message_for_no_site() {
    let text = FinalMessage::new(&[]).to_commit();
    assert_eq!(
        text,
        "just background magic 🧽🔮🧹\n\ncleanup or updating meta files"
    );
}

#[test]
fn commit_message_for_one_site() {
    let text = FinalMessage::example_single().to_commit();
    assert_eq!(
        text,
        "🌐👀 edjopato.de

- https://edjopato.de/post/
"
    );
}

#[test]
fn commit_message_for_two_same_domain_sites() {
    let text = FinalMessage::example_same().to_commit();
    assert_eq!(
        text,
        "🌐👀 edjopato.de

- https://edjopato.de/
- https://edjopato.de/post/
"
    );
}

#[test]
fn commit_message_for_two_different_domain_sites() {
    let text = FinalMessage::example_different().to_commit();
    assert_eq!(
        text,
        "🌐👀 stalked 2 website changes

- https://edjopato.de/post/
- https://foo.bar/
"
    );
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
    let text = FinalMessage::example_same()
        .into_mustache_data(Some("1234abc".into()))
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
    let text = FinalMessage::example_different()
        .into_mustache_data(Some("1234abc".into()))
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
    let text = FinalMessage::example_single()
        .into_mustache_data(None)
        .apply_to_template(None)
        .unwrap();
    assert_eq!(
        text,
        "edjopato.de changed

- https://edjopato.de/post/"
    );
}
