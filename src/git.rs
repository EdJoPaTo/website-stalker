// the git2 crate requires openssl which is annoying to cross compile -> require git to be installed on host

use std::path::Path;
use std::process::{Command, ExitStatus};

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
    let status = Command::new("git").arg("add").arg(path).status()?;
    result_from_status(status, "add")
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
        .arg("commit")
        .arg("--no-gpg-sign")
        .arg("--author")
        .arg("website-stalker <website-stalker-git-commit@edjopato.de>")
        .arg("-m")
        .arg(message)
        .status()?;
    result_from_status(status, "commit")
}

pub fn diff(additional_args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("diff")
        .args(additional_args)
        .status()?;
    result_from_status(status, "diff")
}

pub fn reset() -> anyhow::Result<()> {
    let status = Command::new("git").arg("reset").status()?;
    result_from_status(status, "reset")
}

pub fn status_short() -> anyhow::Result<()> {
    let status = Command::new("git").arg("status").arg("--short").status()?;
    result_from_status(status, "status")
}
