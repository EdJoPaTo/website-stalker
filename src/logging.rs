use std::env;

fn is_gha() -> bool {
    env::var("GITHUB_ACTIONS").is_ok()
}

pub fn error(message: &str) {
    if is_gha() {
        println!("::error file=website-stalker.yaml::{}", message);
    } else {
        println!("ERROR: {}", message);
    }
}
