use lazy_regex::{lazy_regex, Lazy, Regex};
use url::Url;

pub fn basename(url: &Url) -> String {
    static NON_ALPHANUM: Lazy<Regex> = lazy_regex!(r"[^a-zA-Z\d]+");

    let host_part = url.domain().map_or_else(
        || url.host_str().expect("url has a host").to_string(),
        |domain| {
            domain
                .trim_start_matches("www.")
                .split('.')
                .rev()
                .collect::<Vec<_>>()
                .join("-")
        },
    );

    let path = url.path();
    let query = url.query().unwrap_or_default();

    let raw = format!("{host_part}-{path}-{query}");
    let only_ascii = NON_ALPHANUM.replace_all(&raw, "-");
    only_ascii.trim_matches('-').to_string()
}

#[cfg(test)]
/// test base name
fn tb(url: &str) -> String {
    let url = Url::parse(url).unwrap();
    println!("{url}");
    basename(&url)
}

#[test]
fn examples() {
    assert_eq!(tb("https://edjopato.de/"), "de-edjopato");
    assert_eq!(tb("https://edjopato.de/post/"), "de-edjopato-post");
}

#[test]
fn query_does_matter() {
    assert_eq!(
        tb("http://edjopato.de/?something=true"),
        "de-edjopato-something-true",
    );
}

#[test]
fn scheme_doesnt_matter() {
    assert_eq!(tb("http://edjopato.de/"), tb("https://edjopato.de/"));
}

#[test]
fn fragment_doesnt_matter() {
    assert_eq!(
        tb("http://edjopato.de/"),
        tb("https://edjopato.de/#whatever"),
    );
}

#[test]
fn ending_slash_doesnt_matter() {
    assert_eq!(tb("https://edjopato.de/"), tb("https://edjopato.de"));
    assert_eq!(
        tb("https://edjopato.de/post/"),
        tb("https://edjopato.de/post"),
    );
}

#[test]
fn extension_is_still_in_basename() {
    assert_eq!(tb("http://edjopato.de/robot.txt"), "de-edjopato-robot-txt");
    assert_eq!(
        tb("http://edjopato.de/robot.html"),
        "de-edjopato-robot-html",
    );
}

#[test]
fn domain_prefix_www_doesnt_matter() {
    assert_eq!(tb("https://edjopato.de/"), tb("https://www.edjopato.de/"));
}

#[test]
fn works_with_ipv4() {
    assert_eq!(tb("http://127.0.0.1/test/"), "127-0-0-1-test");
}

#[test]
fn works_with_ipv6() {
    assert_eq!(tb("http://[::1]/test/"), "1-test");
}
