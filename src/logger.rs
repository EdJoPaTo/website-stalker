use std::env;

use once_cell::sync::Lazy;

static GHA: Lazy<bool> = Lazy::new(|| env::var_os("GITHUB_ACTIONS").is_some());

pub fn error(message: &str) {
    if *GHA {
        println!("::error file=website-stalker.yaml::{message}");
    } else {
        eprintln!("ERROR: {message}");
    }
}

pub fn warn(message: &str) {
    if *GHA {
        println!("::warning file=website-stalker.yaml::{message}");
    } else {
        eprintln!("WARN: {message}");
    }
}

pub fn info(message: &str) {
    eprintln!("INFO: {message}");
}
