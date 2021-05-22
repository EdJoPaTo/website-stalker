use std::env;

fn is_gha() -> bool {
    env::var_os("GITHUB_ACTIONS").is_some()
}

pub fn error(message: &str) {
    if is_gha() {
        println!("::error file=website-stalker.yaml::{}", message);
    } else {
        println!("ERROR: {}", message);
    }
}

pub fn begin_group(title: &str) {
    if is_gha() {
        println!("::group::{}", title);
    }
}

pub fn end_group() {
    if is_gha() {
        println!("::endgroup::");
    }
}
