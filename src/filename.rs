use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

pub fn basename(url: &Url) -> String {
    static NON_ALPHANUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-zA-Z\d]+").unwrap());

    let domain = url.domain().expect("domain needed");
    let path = url.path();
    let query = url.query().unwrap_or_default();

    let domain_part = domain
        .trim_start_matches("www.")
        .split('.')
        .rev()
        .collect::<Vec<_>>()
        .join("-");
    let raw = format!("{domain_part}-{path}-{query}");
    let only_ascii = NON_ALPHANUM.replace_all(&raw, "-");
    only_ascii.trim_matches('-').to_string()
}

#[cfg(test)]
/// test basename
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
