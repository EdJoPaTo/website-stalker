use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use git2::{IndexAddOption, Repository, Signature};

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

pub struct Repo {
    repo: Repository,
}

impl Repo {
    pub fn new() -> Result<Self, git2::Error> {
        let repo = Repository::open_from_env()?;

        if repo.is_bare() {
            panic!("Repo needs a work tree. This does not work with bare repos.");
        }

        Ok(Self { repo })
    }

    pub fn add<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let workdir = self.repo.workdir().unwrap();
        let abs = fs::canonicalize(path)?;
        let relative = abs.strip_prefix(workdir)?;

        let mut index = self.repo.index()?;
        index.add_all(relative, IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub fn cleanup(&self, path: &str) -> anyhow::Result<()> {
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

    pub fn commit(&self, message: &str) -> anyhow::Result<()> {
        let signature = Signature::now(GIT_COMMIT_AUTHOR_NAME, GIT_COMMIT_AUTHOR_EMAIL)?;
        let tree = self.repo.find_tree(self.repo.index()?.write_tree()?)?;
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub fn diff(&self, additional_args: &[&str]) -> anyhow::Result<()> {
        let status = Command::new("git")
            .arg("--no-pager")
            .arg("diff")
            .args(additional_args)
            .status()?;
        result_from_status(status, "diff")
    }

    pub fn reset(&self) -> anyhow::Result<()> {
        let oid = self
            .repo
            .head()?
            .target()
            .ok_or_else(|| anyhow::anyhow!("HEAD reference is not a direct reference"))?;
        let obj = self.repo.find_object(oid, None)?;
        self.repo.reset(&obj, git2::ResetType::Mixed, None)?;
        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub fn status_short(&self) -> anyhow::Result<()> {
        let status = Command::new("git")
            .arg("--no-pager")
            .arg("status")
            .arg("--short")
            .status()?;
        result_from_status(status, "status")
    }
}
