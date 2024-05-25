use std::fmt::Write;

use url::Url;

pub fn commit_message(changed_urls: &[Url]) -> String {
    let mut sites = changed_urls.iter().collect::<Vec<_>>();
    sites.sort_unstable();
    sites.dedup();

    let mut hosts = sites
        .iter()
        .filter_map(|url| url.host_str())
        .collect::<Vec<_>>();
    hosts.dedup();

    let mut text = match hosts.as_slice() {
        [] => "just background magic 🧽🔮🧹\n\ncleanup or updating meta files".to_owned(),
        [single] => format!("🌐👀 {single}\n\n"),
        _ => format!("🌐👀 stalked {} website changes\n\n", sites.len()),
    };
    for site in sites {
        _ = writeln!(&mut text, "- {site}");
    }
    text
}

#[test]
fn commit_message_for_no_site() {
    assert_eq!(
        commit_message(&[]),
        "just background magic 🧽🔮🧹\n\ncleanup or updating meta files"
    );
}

#[test]
fn commit_message_for_one_site() {
    let urls = [Url::parse("https://edjopato.de/post/").unwrap()];
    assert_eq!(
        commit_message(&urls),
        "🌐👀 edjopato.de

- https://edjopato.de/post/
"
    );
}

#[test]
fn commit_message_for_two_same_domain_sites() {
    let urls = [
        Url::parse("https://edjopato.de/").unwrap(),
        Url::parse("https://edjopato.de/post/").unwrap(),
    ];
    assert_eq!(
        commit_message(&urls),
        "🌐👀 edjopato.de

- https://edjopato.de/
- https://edjopato.de/post/
"
    );
}

#[test]
fn commit_message_for_two_different_domain_sites() {
    let urls = [
        Url::parse("https://edjopato.de/post/").unwrap(),
        Url::parse("https://foo.bar/").unwrap(),
    ];
    assert_eq!(
        commit_message(&urls),
        "🌐👀 stalked 2 website changes

- https://edjopato.de/post/
- https://foo.bar/
"
    );
}
