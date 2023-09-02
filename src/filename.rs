use lazy_regex::{lazy_regex, Lazy, Regex};
use url::Url;

pub fn domainfolder(url: &Url) -> String {
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
    let port = url.port().map(|o| o.to_string()).unwrap_or_default();
    alphanum(&format!("{host_part}-{port}"))
}

pub fn filename(url: &Url) -> String {
    let path = url.path();
    let query = url.query().unwrap_or_default();
    let output = alphanum(&format!("{path}-{query}"));
    if output.is_empty() {
        "index".to_string()
    } else {
        output
    }
}

fn alphanum(str: &str) -> String {
    static NON_ALPHANUM: Lazy<Regex> = lazy_regex!(r"[^a-zA-Z\d]+");
    NON_ALPHANUM
        .replace_all(str, "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
/// test base name
fn tb(url: &str) -> String {
    let url = Url::parse(url).expect("url is valid");
    println!("{url}");
    let folder = domainfolder(&url);
    let file = filename(&url);
    format!("{folder}/{file}")
}

#[test]
fn examples() {
    assert_eq!(tb("https://edjopato.de/"), "de-edjopato/index");
    assert_eq!(tb("https://edjopato.de/post/"), "de-edjopato/post");
}

#[test]
fn query_does_matter() {
    assert_eq!(
        tb("http://edjopato.de/?something=true"),
        "de-edjopato/something-true",
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
    assert_eq!(tb("http://edjopato.de/robot.txt"), "de-edjopato/robot-txt");
    assert_eq!(
        tb("http://edjopato.de/robot.html"),
        "de-edjopato/robot-html",
    );
}

#[test]
fn domain_prefix_www_doesnt_matter() {
    assert_eq!(tb("https://edjopato.de/"), tb("https://www.edjopato.de/"));
}

#[test]
fn works_with_ipv4() {
    assert_eq!(tb("http://127.0.0.1/test/"), "127-0-0-1/test");
}

#[test]
fn works_with_ipv4_with_port() {
    assert_eq!(tb("http://127.0.0.1:12345/test/"), "127-0-0-1-12345/test");
}

#[test]
fn works_with_ipv6() {
    assert_eq!(tb("http://[::1]/test/"), "1/test");
}

#[test]
#[should_panic = "url is valid"]
fn fails_on_ipv6_with_interface() {
    assert_eq!(tb("http://[fe80::1234%eth0]/test/"), "fe80-1234-eth0/test");
}
