// wait for release to be breaking
#![allow(deprecated)]

use std::fmt::Write;

use url::Url;

pub const DEFAULT_NOTIFICATION_TEMPLATE: &str = "
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
";

#[derive(serde::Serialize)]
pub struct FinalMessage {
    #[deprecated = "use hosts"]
    domains: Vec<String>,
    hosts: Vec<String>,
    sites: Vec<String>,
}

#[derive(serde::Serialize)]
struct MustacheData {
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
        for s in &self.sites {
            _ = writeln!(&mut text, "- {s}");
        }
        text
    }

    fn into_mustache_data(self, commit: Option<String>) -> MustacheData {
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

    pub fn into_notification(
        self,
        template: Option<&str>,
        commit: Option<String>,
    ) -> anyhow::Result<String> {
        let template = mustache::compile_str(template.unwrap_or(DEFAULT_NOTIFICATION_TEMPLATE))?;
        let data = self.into_mustache_data(commit);
        let message = template.render_to_string(&data)?;
        Ok(message.trim().to_owned())
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

    pub fn validate_template(template: &str) -> anyhow::Result<()> {
        let template = Some(template);
        let any_empty = [
            Self::example_single().into_notification(template, Some("666".into()))?,
            Self::example_single().into_notification(template, None)?,
            Self::example_different().into_notification(template, Some("666".into()))?,
            Self::example_different().into_notification(template, None)?,
            Self::example_same().into_notification(template, Some("666".into()))?,
            Self::example_same().into_notification(template, None)?,
        ]
        .iter()
        .any(std::string::String::is_empty);

        if any_empty {
            Err(anyhow::anyhow!("template produced empty notification text"))
        } else {
            Ok(())
        }
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
    FinalMessage::validate_template("Hello {{name}}").unwrap();
}

#[test]
fn default_template_is_valid() {
    FinalMessage::validate_template(DEFAULT_NOTIFICATION_TEMPLATE).unwrap();
}

#[test]
#[should_panic = "unclosed tag"]
fn invalid_template_isnt_valid() {
    FinalMessage::validate_template("Hello World {{").unwrap();
}

#[test]
fn notification_message_for_two_same_domain_sites() {
    let text = FinalMessage::example_same()
        .into_notification(None, Some("1234abc".into()))
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
        .into_notification(None, Some("1234abc".into()))
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
        .into_notification(None, None)
        .unwrap();
    assert_eq!(
        text,
        "edjopato.de changed

- https://edjopato.de/post/"
    );
}
