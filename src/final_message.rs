use crate::site::Site;
use crate::ChangeKind;

pub const DEFAULT_NOTIFICATION_TEMPLATE: &str = "
{{#singledomain}}
{{singledomain}} changed
{{/singledomain}}
{{^singledomain}}
{{siteamount}} websites changed
{{/singledomain}}

{{#sites}}
{{.}}
{{/sites}}

{{#commit}}
See {{.}}
{{/commit}}
";

#[derive(serde::Serialize)]
pub struct FinalMessage {
    domains: Vec<String>,
    sites: Vec<String>,
}

#[derive(serde::Serialize)]
struct MustacheData {
    commit: Option<String>,
    singledomain: Option<String>,
    siteamount: usize,

    #[serde(flatten)]
    msg: FinalMessage,
}

impl FinalMessage {
    pub fn new(changed_sites: &[Site]) -> Self {
        let mut domains = changed_sites
            .iter()
            .filter_map(|(_, site)| site.url.domain())
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        domains.sort_unstable();
        domains.dedup();

        let mut sites = changed_sites
            .iter()
            .map(handled_site_line)
            .collect::<Vec<_>>();
        sites.sort();
        sites.dedup();

        Self { domains, sites }
    }

    pub fn to_commit(&self) -> String {
        let head = match self.domains.as_slice() {
            [] => "just background magic \u{1f9fd}\u{1f52e}\u{1f9f9}\n\ncleanup or updating meta files"
                .to_string(), // 🧽🔮🧹
            [single] => format!("\u{1f310}\u{1f440} {}", single), // 🌐👀
            _ => format!(
                "\u{1f310}\u{1f440} stalked {} website changes", // 🌐👀
                self.sites.len()
            ),
        };
        let body = self.sites.join("\n");

        let text = format!("{}\n\n{}", head, body);
        text.trim().to_string()
    }

    fn into_mustache_data(self, commit: Option<String>) -> MustacheData {
        let singledomain = if let [single] = self.domains.as_slice() {
            Some(single.to_string())
        } else {
            None
        };

        MustacheData {
            commit,
            singledomain,
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
        Ok(message.trim().to_string())
    }

    fn example_single() -> Self {
        Self::new(&[(
            ChangeKind::Changed,
            Site {
                url: url::Url::parse("https://edjopato.de/post/").unwrap(),
                options: crate::site::Options {
                    accept_invalid_certs: false,
                    editors: vec![],
                },
            },
        )])
    }

    fn example_different() -> Self {
        Self::new(&[
            (
                ChangeKind::Changed,
                Site {
                    url: url::Url::parse("https://edjopato.de/post/").unwrap(),
                    options: crate::site::Options {
                        accept_invalid_certs: false,
                        editors: vec![],
                    },
                },
            ),
            (
                ChangeKind::Changed,
                Site {
                    url: url::Url::parse("https://foo.bar/").unwrap(),
                    options: crate::site::Options {
                        accept_invalid_certs: false,
                        editors: vec![],
                    },
                },
            ),
        ])
    }

    fn example_same() -> Self {
        Self::new(&[
            (
                ChangeKind::Changed,
                Site {
                    url: url::Url::parse("https://edjopato.de/").unwrap(),
                    options: crate::site::Options {
                        accept_invalid_certs: false,
                        editors: vec![],
                    },
                },
            ),
            (
                ChangeKind::Changed,
                Site {
                    url: url::Url::parse("https://edjopato.de/post/").unwrap(),
                    options: crate::site::Options {
                        accept_invalid_certs: false,
                        editors: vec![],
                    },
                },
            ),
        ])
    }

    pub fn validate_template(template: &str) -> anyhow::Result<()> {
        let template = Some(template);
        let any_empty = vec![
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

fn handled_site_line(handled_site: &(ChangeKind, Site)) -> String {
    let (change_kind, site) = handled_site;
    let letter = match change_kind {
        ChangeKind::Init => 'A',
        ChangeKind::Changed => 'M',
        ChangeKind::ContentSame => unreachable!(),
    };
    format!("{} {}", letter, site.url.as_str())
}

#[test]
fn commit_message_for_no_site() {
    let text = FinalMessage::new(&[]).to_commit();
    assert_eq!(
        text,
        "just background magic \u{1f9fd}\u{1f52e}\u{1f9f9}\n\ncleanup or updating meta files"
    );
}

#[test]
fn commit_message_for_one_site() {
    let text = FinalMessage::example_single().to_commit();
    assert_eq!(
        text,
        "\u{1f310}\u{1f440} edjopato.de

M https://edjopato.de/post/"
    );
}

#[test]
fn commit_message_for_two_same_domain_sites() {
    let text = FinalMessage::example_same().to_commit();
    assert_eq!(
        text,
        "\u{1f310}\u{1f440} edjopato.de

M https://edjopato.de/
M https://edjopato.de/post/"
    );
}

#[test]
fn commit_message_for_two_different_domain_sites() {
    let text = FinalMessage::example_different().to_commit();
    assert_eq!(
        text,
        "\u{1f310}\u{1f440} stalked 2 website changes

M https://edjopato.de/post/
M https://foo.bar/"
    );
}

#[test]
fn default_template_is_valid() {
    FinalMessage::validate_template(DEFAULT_NOTIFICATION_TEMPLATE).unwrap();
}

#[test]
fn notification_message_for_two_same_domain_sites() {
    let text = FinalMessage::example_same()
        .into_notification(None, Some("1234abc".into()))
        .unwrap();
    assert_eq!(
        text,
        "edjopato.de changed

M https://edjopato.de/
M https://edjopato.de/post/

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

M https://edjopato.de/post/
M https://foo.bar/

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

M https://edjopato.de/post/"
    );
}
