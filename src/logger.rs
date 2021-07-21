use std::env;

fn is_gha() -> bool {
    env::var_os("GITHUB_ACTIONS").is_some()
}

pub fn error(message: &str) {
    if is_gha() {
        println!("::error file=website-stalker.yaml::{}", message);
    } else {
        eprintln!("ERROR: {}", message);
    }
}

pub fn warn(message: &str) {
    if is_gha() {
        println!("::warning file=website-stalker.yaml::{}", message);
    } else {
        eprintln!("WARN: {}", message);
    }
}

pub fn info(message: &str) {
    eprintln!("INFO: {}", message);
}
