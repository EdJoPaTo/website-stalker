// the git2 crate requires openssl which is annoying to cross compile -> require git to be installed on host

use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const GIT_COMMIT_AUTHOR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    "<website-stalker-git-commit@edjopato.de>"
);

fn result_from_status(status: ExitStatus, command: &'static str) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "failed git {} with status code {}",
            command,
            status
        ))
    }
}

pub fn is_repo() -> bool {
    Path::new(".git/HEAD").exists()
}

pub fn add(path: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("add")
        .arg(path)
        .status()?;
    result_from_status(status, "add")
}

pub fn cleanup(path: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("clean")
        .arg("--force")
        .arg("--quiet")
        .arg("-x") // remove untracked files
        .arg(path)
        .status()?;
    result_from_status(status, "clean")?;

    let status = Command::new("git")
        .arg("--no-pager")
        .arg("checkout")
        .arg("--quiet")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    drop(status);

    Ok(())
}

pub fn commit(message: &str) -> anyhow::Result<()> {
    let message = format!(
        "{}/{}: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        message
    );
    #[cfg(debug_assertions)]
    println!("commit message length is {}/50 {}", message.len(), message);
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("commit")
        .arg("--no-gpg-sign")
        .arg("--author")
        .arg(GIT_COMMIT_AUTHOR)
        .arg("-m")
        .arg(message)
        .status()?;
    result_from_status(status, "commit")
}

pub fn diff(additional_args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("diff")
        .args(additional_args)
        .status()?;
    result_from_status(status, "diff")
}

pub fn reset() -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("reset")
        .status()?;
    result_from_status(status, "reset")
}

pub fn status_short() -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("--no-pager")
        .arg("status")
        .arg("--short")
        .status()?;
    result_from_status(status, "status")
}
