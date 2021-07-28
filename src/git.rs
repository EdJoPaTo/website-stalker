use std::fs;
use std::path::{Path, PathBuf};

use git2::{Diff, DiffOptions, IndexAddOption, Repository, RepositoryInitOptions, Signature};

const GIT_COMMIT_AUTHOR_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const GIT_COMMIT_AUTHOR_EMAIL: &str = "website-stalker-git-commit@edjopato.de";

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

    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, git2::Error> {
        let repo = Repository::init_opts(path, RepositoryInitOptions::new().initial_head("main"))?;
        Ok(Self { repo })
    }

    fn relative_to_repo<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<PathBuf> {
        let workdir = self.repo.workdir().unwrap();
        let abs = fs::canonicalize(path)?;
        let relative = abs.strip_prefix(workdir)?;
        Ok(relative.to_owned())
    }

    pub fn add<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let relative = self.relative_to_repo(path)?;

        let mut index = self.repo.index()?;
        index.add_all(relative.as_path(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<(), git2::Error> {
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

    fn diff(&self) -> Result<Diff, git2::Error> {
        let mut opts = DiffOptions::new();
        opts.include_untracked(true);
        self.repo.diff_tree_to_workdir_with_index(
            self.repo.head()?.peel_to_tree().ok().as_ref(),
            Some(&mut opts),
        )
    }

    pub fn is_something_modified(&self) -> Result<bool, git2::Error> {
        Ok(self.diff()?.stats()?.files_changed() > 0)
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    fn simple_command<P: AsRef<Path>>(dir: P, command: &str) -> anyhow::Result<String> {
        let output = std::process::Command::new("bash")
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

    fn init_test_env() -> anyhow::Result<(tempfile::TempDir, Repo)> {
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

    fn println_command<P: AsRef<Path>>(dir: P, command: &str) {
        println!("# {}", command);
        match simple_command(dir, command) {
            Ok(output) => println!("{}", output),
            Err(err) => println!("{}", err),
        };
    }

    fn overview<P: AsRef<Path>>(dir: P) {
        println_command(&dir, "pwd");
        println_command(&dir, "ls -al");
        println_command(&dir, "git status --short");
        println_command(&dir, "git log");
    }

    #[test]
    fn init_works() -> anyhow::Result<()> {
        let tempdir = tempfile::Builder::new()
            .prefix("website-stalker-testing-")
            .tempdir()?;
        let dir = tempdir.path();
        Repo::init(dir)?;
        assert_eq!(simple_command(dir, "git status --short")?, "");
        simple_command(dir, "touch bla.txt")?;
        assert_eq!(simple_command(dir, "git status --short")?, "?? bla.txt");
        Ok(())
    }

    #[test]
    fn init_default_branch_is_main() -> anyhow::Result<()> {
        let tempdir = tempfile::Builder::new()
            .prefix("website-stalker-testing-")
            .tempdir()?;
        let dir = tempdir.path();
        Repo::init(dir)?;
        assert_eq!(simple_command(dir, "git branch --show-current")?, "main");
        Ok(())
    }

    #[test]
    fn add_works() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        simple_command(dir, "touch bla.txt")?;
        assert_eq!(simple_command(dir, "git status --short")?, "?? bla.txt");
        repo.add(dir.join("bla.txt"))?;
        assert_eq!(simple_command(dir, "git status --short")?, "A  bla.txt");
        Ok(())
    }

    #[test]
    fn commit_commits() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
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

    #[test]
    fn is_something_modified_untracked() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        assert!(!repo.is_something_modified()?);

        simple_command(dir, "echo stuff > bla.txt")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git add bla.txt")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git reset")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git clean -xdf")?;
        assert!(!repo.is_something_modified()?);
        Ok(())
    }

    #[test]
    fn is_something_modified_changed() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        assert!(!repo.is_something_modified()?);

        simple_command(dir, "echo stuff > bla.txt")?;
        simple_command(dir, "git add bla.txt")?;
        simple_command(dir, "git commit -m bla")?;
        assert!(!repo.is_something_modified()?);

        simple_command(dir, "echo foo > bla.txt")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git add bla.txt")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git reset")?;
        assert!(repo.is_something_modified()?);
        simple_command(dir, "git checkout bla.txt")?;
        assert!(!repo.is_something_modified()?);
        Ok(())
    }
}
