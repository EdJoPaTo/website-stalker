use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use git2::{IndexAddOption, IntoCString, Repository, Signature};

const GIT_COMMIT_AUTHOR_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const GIT_COMMIT_AUTHOR_EMAIL: &str = "website-stalker-git-commit@edjopato.de";

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

pub fn add<I, S>(paths: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: IntoCString,
{
    let repo = Repository::open(&Path::new("."))?;
    let mut index = repo.index()?;
    index.add_all(paths, IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
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
    let repo = Repository::open(&Path::new("."))?;
    let signature = Signature::now(GIT_COMMIT_AUTHOR_NAME, GIT_COMMIT_AUTHOR_EMAIL)?;
    let tree = repo.find_tree(repo.index()?.write_tree()?)?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;
    Ok(())
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
