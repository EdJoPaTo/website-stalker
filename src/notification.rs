use std::collections::BTreeMap;
use std::fmt::Write;

use url::Url;

fn generate_change_lines(mut changed: Vec<Url>) -> String {
    debug_assert!(!changed.is_empty(), "no change no notification");

    changed.sort_unstable();
    changed.dedup();

    let mut changed_hosts = BTreeMap::<String, Vec<Url>>::new();
    for url in changed {
        if let Some(host) = url.host_str() {
            let host = host.to_owned();
            changed_hosts.entry(host).or_default().push(url);
        }
    }

    let mut text = String::new();

    for urls in changed_hosts.values() {
        if let [single_url] = &**urls {
            _ = writeln!(text, "- {single_url}");
        }
    }

    for urls in changed_hosts.values() {
        if urls.len() > 1 {
            _ = writeln!(text);
            for url in urls {
                _ = writeln!(text, "- {url}");
            }
        }
    }

    text.trim().to_owned()
}

fn generate_commit_part(commit: Option<String>, template: Option<String>) -> Option<String> {
    if let Some(template) = template {
        commit.map(|commit| {
            if template.contains("{commit}") {
                template.replace("{commit}", &commit)
            } else {
                format!("{template}{commit}")
            }
        })
    } else {
        commit
    }
}

pub fn generate_text(
    commit: Option<String>,
    commit_template: Option<String>,
    changed: Vec<Url>,
) -> String {
    let change_lines = generate_change_lines(changed);

    if let Some(commit) = generate_commit_part(commit, commit_template) {
        format!("{change_lines}\n\n{commit}")
    } else {
        change_lines
    }
}

#[test]
fn e2e_with_commit() {
    let result = generate_text(
        Some("1234abc".to_owned()),
        None,
        vec![Url::parse("https://edjopato.de/").unwrap()],
    );
    assert_eq!(result, "- https://edjopato.de/\n\n1234abc");
}

#[test]
fn e2e_without_commit() {
    let result = generate_text(
        None,
        None,
        vec![Url::parse("https://edjopato.de/").unwrap()],
    );
    assert_eq!(result, "- https://edjopato.de/");
}

#[cfg(test)]
mod change_lines_tests {
    use super::*;

    fn test(changed: &[&str], expected: &str) {
        let changed = changed
            .iter()
            .map(|url| url.parse::<Url>().expect("test input should be valid URL"))
            .collect();
        let lines = generate_change_lines(changed);
        assert_eq!(lines, expected);
    }

    #[test]
    fn single() {
        test(
            &["https://edjopato.de/post/"],
            "- https://edjopato.de/post/",
        );
    }

    #[test]
    fn multiple_on_single_host() {
        test(
            &["https://edjopato.de/", "https://edjopato.de/post/"],
            "- https://edjopato.de/
- https://edjopato.de/post/",
        );
    }

    #[test]
    fn multiple_per_host() {
        test(
            &[
                "https://edjopato.de/",
                "https://edjopato.de/post/",
                "https://example.com/",
                "https://example.com/path",
            ],
            "- https://edjopato.de/
- https://edjopato.de/post/

- https://example.com/
- https://example.com/path",
        );
    }

    #[test]
    fn different_hosts() {
        test(
            &["https://edjopato.de/post/", "https://example.com/"],
            "- https://edjopato.de/post/
- https://example.com/",
        );
    }

    #[test]
    fn mixed() {
        test(
            &[
                "https://edjopato.de/",
                "https://edjopato.de/post/",
                "https://example.com/",
            ],
            "- https://example.com/

- https://edjopato.de/
- https://edjopato.de/post/",
        );
    }
}

#[cfg(test)]
mod commit_template_tests {
    use super::*;

    fn test(commit: Option<&str>, template: Option<&str>, expected: Option<&str>) {
        let expected = expected.map(ToOwned::to_owned);
        let part = generate_commit_part(
            commit.map(ToOwned::to_owned),
            template.map(ToOwned::to_owned),
        );
        assert_eq!(part, expected);
    }

    #[test]
    fn empty_commit_always_empty() {
        test(None, None, None);
        test(None, Some("prefix"), None);
    }

    #[test]
    fn prefix() {
        test(Some("1234abc"), Some("prefix/"), Some("prefix/1234abc"));
    }

    #[test]
    fn replace() {
        test(
            Some("1234abc"),
            Some("some {commit} text"),
            Some("some 1234abc text"),
        );
    }
}
