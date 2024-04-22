/*!
GitHub related logic

# Documentation
- <https://docs.github.com/en/actions/learn-github-actions/variables#default-environment-variables>
- <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions>
*/

use std::env;

use once_cell::sync::Lazy;

pub static IS_RUN_AS_GITHUB_ACTION: Lazy<bool> =
    Lazy::new(|| env::var_os("GITHUB_ACTIONS").is_some());

pub fn error(message: &str) {
    println!("::error file=website-stalker.yaml::{message}");
}

pub fn warning(message: &str) {
    println!("::warning file=website-stalker.yaml::{message}");
}

/// See [`crate::notification`]
pub fn commit_prefix() -> Option<String> {
    let server = env::var("GITHUB_SERVER_URL").ok()?;
    let repo = env::var("GITHUB_REPOSITORY").ok()?;
    Some(format!("{server}/{repo}/commit/"))
}
