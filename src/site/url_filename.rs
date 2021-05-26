use regex::Regex;
use url::Url;

pub fn format(url: &Url, extension: &str) -> String {
    let re = Regex::new("[^a-zA-Z\\d]+").unwrap();
    let domain = url.domain().expect("domain needed");
    let path = url.path().trim_end_matches(extension);

    let domain_part = domain
        .trim_start_matches("www.")
        .split('.')
        .rev()
        .collect::<Vec<_>>()
        .join("-");
    let raw = format!("{}-{}", domain_part, path);
    let only_ascii = re.replace_all(&raw, "-");
    let trimmed = only_ascii.trim_matches('-');
    format!("{}.{}", trimmed, extension)
}

#[cfg(test)]
/// test format
fn tf(url: &str, extension: &str) -> String {
    let url = Url::parse(url).unwrap();
    println!("{}", url);
    format(&url, extension)
}

#[test]
fn examples() {
    assert_eq!(tf("https://edjopato.de/", "html"), "de-edjopato.html");

    assert_eq!(
        tf("https://edjopato.de/post/", "html"),
        "de-edjopato-post.html"
    );
}

#[test]
fn scheme_doesnt_matter() {
    assert_eq!(
        tf("http://edjopato.de/", "html"),
        tf("https://edjopato.de/", "html"),
    );
}

#[test]
fn fragment_doesnt_matter() {
    let expect = tf("http://edjopato.de/", "html");
    let actual = tf("https://edjopato.de/#whatever", "html");
    assert_eq!(expect, actual);
}

#[test]
fn ending_slash_doesnt_matter() {
    assert_eq!(
        tf("https://edjopato.de/", "html"),
        tf("https://edjopato.de", "html"),
    );

    assert_eq!(
        tf("https://edjopato.de/post/", "html"),
        tf("https://edjopato.de/post", "html"),
    );
}

#[test]
fn extension_does_not_duplicate() {
    assert_eq!(
        tf("http://edjopato.de/robot.txt", "txt"),
        "de-edjopato-robot.txt",
    );

    assert_eq!(
        tf("http://edjopato.de/robot.html", "txt"),
        "de-edjopato-robot-html.txt",
    );
}

#[test]
fn domain_prefix_www_doesnt_matter() {
    assert_eq!(
        tf("https://edjopato.de/", "html"),
        tf("https://www.edjopato.de/", "html"),
    );
}
