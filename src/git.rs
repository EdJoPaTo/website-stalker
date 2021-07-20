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

    pub fn cleanup(&self, path: &str) -> anyhow::Result<()> {
        let status = Command::new("git")
            .current_dir(self.repo.path().parent().unwrap())
            .arg("--no-pager")
            .arg("clean")
            .arg("--force")
            .arg("--quiet")
            .arg("-x") // remove untracked files
            .arg(path)
            .status()?;
        result_from_status(status, "clean")?;

        let status = Command::new("git")
            .current_dir(self.repo.path().parent().unwrap())
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

    pub fn diff(&self, additional_args: &[&str]) -> anyhow::Result<()> {
        let status = Command::new("git")
            .current_dir(self.repo.path().parent().unwrap())
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

    pub fn status_short(&self) -> anyhow::Result<()> {
        let status = Command::new("git")
            .current_dir(self.repo.path().parent().unwrap())
            .arg("--no-pager")
            .arg("status")
            .arg("--short")
            .status()?;
        result_from_status(status, "status")
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
fn simple_command<P: AsRef<Path>>(dir: P, command: &str) -> anyhow::Result<String> {
    let output = Command::new("bash")
        .current_dir(dir)
        .arg("-c")
        .arg(command)
        .output()?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.trim().to_string())
    } else {
        Err(anyhow::anyhow!(
            "failed command \"{}\" with status code {}",
            command,
            output.status
        ))
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
fn init_repo() -> anyhow::Result<(tempfile::TempDir, Repo)> {
    let tempdir = tempfile::Builder::new()
        .prefix("website-stalker-testing-")
        .tempdir()?;
    let dir = tempdir.path();

    let repo = Repository::init(&dir)?;
    let repo = Repo { repo };
    simple_command(dir, "git config user.email bla@blubb.de")?;
    simple_command(dir, "git config user.name Bla")?;
    simple_command(dir, "git commit -m init --allow-empty")?;
    Ok((tempdir, repo))
}

#[cfg(target_os = "linux")]
#[cfg(test)]
fn println_command<P: AsRef<Path>>(dir: P, command: &str) {
    println!("# {}", command);
    match simple_command(dir, command) {
        Ok(output) => println!("{}", output),
        Err(err) => println!("{}", err),
    };
}

#[cfg(target_os = "linux")]
#[cfg(test)]
fn overview<P: AsRef<Path>>(dir: P) {
    println_command(&dir, "pwd");
    println_command(&dir, "ls -al");
    println_command(&dir, "git status --short");
    println_command(&dir, "git log");
}

#[cfg(target_os = "linux")]
#[test]
fn reset_works() -> anyhow::Result<()> {
    let (tempdir, repo) = init_repo()?;
    let dir = tempdir.path();
    simple_command(dir, "touch bla.txt")?;
    simple_command(dir, "git add bla.txt")?;
    overview(dir);
    assert_eq!(simple_command(dir, "git status --short")?, "A  bla.txt");
    repo.reset()?;
    assert_eq!(simple_command(dir, "git status --short")?, "?? bla.txt");
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn cleanup_resets_existing_file() -> anyhow::Result<()> {
    let (tempdir, repo) = init_repo()?;
    let dir = tempdir.path();
    simple_command(dir, "mkdir foo")?;
    simple_command(dir, "echo stuff > foo/bar.txt")?;
    assert_eq!(simple_command(dir, "du -b foo/bar.txt")?, "6\tfoo/bar.txt");
    simple_command(dir, "git add foo")?;
    simple_command(dir, "git commit -m bla")?;
    simple_command(dir, "echo longstuff > foo/bar.txt")?;
    assert_eq!(simple_command(dir, "du -b foo/bar.txt")?, "10\tfoo/bar.txt");
    repo.cleanup("foo")?;
    overview(dir);
    assert_eq!(simple_command(dir, "du -b foo/bar.txt")?, "6\tfoo/bar.txt");
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn cleanup_removes_superfluous() -> anyhow::Result<()> {
    let (tempdir, repo) = init_repo()?;
    let dir = tempdir.path();
    simple_command(dir, "mkdir foo")?;
    simple_command(dir, "echo stuff > foo/bar.txt")?;
    simple_command(dir, "git add foo")?;
    simple_command(dir, "git commit -m bla")?;
    simple_command(dir, "echo longstuff > foo/other.txt")?;
    assert_eq!(
        simple_command(dir, "git status --short")?,
        "?? foo/other.txt"
    );
    repo.cleanup("foo")?;
    overview(dir);
    assert_eq!(simple_command(dir, "ls foo")?, "bar.txt");
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn cleanup_keeps_outside_changed_file() -> anyhow::Result<()> {
    let (tempdir, repo) = init_repo()?;
    let dir = tempdir.path();
    simple_command(dir, "mkdir foo")?;
    simple_command(dir, "echo stuff > foo/bar.txt")?;
    simple_command(dir, "git add foo")?;
    simple_command(dir, "git commit -m bla")?;
    simple_command(dir, "echo longstuff > other.txt")?;
    assert_eq!(simple_command(dir, "git status --short")?, "?? other.txt");
    repo.cleanup("foo")?;
    overview(dir);
    assert_eq!(simple_command(dir, "git status --short")?, "?? other.txt");
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn commit_commits() -> anyhow::Result<()> {
    let (tempdir, repo) = init_repo()?;
    let dir = tempdir.path();
    simple_command(dir, "echo stuff > bla.txt")?;
    simple_command(dir, "git add bla.txt")?;
    overview(dir);
    assert_eq!(simple_command(dir, "git log")?.lines().count(), 5);
    repo.commit("bla")?;
    overview(dir);
    assert_eq!(simple_command(dir, "git log")?.lines().count(), 11);
    Ok(())
}
