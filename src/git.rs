use std::path::{Path, PathBuf};
use std::process::Command;

const GIT_COMMIT_AUTHOR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " <website-stalker-git-commit@edjopato.de>"
);

fn git_command(dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git").args(args).current_dir(dir).output()?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.trim().to_owned())
    } else {
        Err(anyhow::anyhow!(
            "failed git command \"{}\" with {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

pub struct Repo {
    dir: PathBuf,
}

impl Repo {
    fn git_command(&self, args: &[&str]) -> anyhow::Result<String> {
        git_command(&self.dir, args)
    }

    pub fn new() -> anyhow::Result<Self> {
        let workdir = std::env::current_dir()?;

        let repodir = git_command(&workdir, &["rev-parse", "--show-toplevel"])?;
        let repodir = Path::new(&repodir).canonicalize()?;

        if !repodir.exists() || repodir != workdir {
            anyhow::bail!("not on repository toplevel: {}", repodir.display());
        }

        Ok(Self { dir: repodir })
    }

    pub fn init(path: &Path) {
        git_command(path, &["init"]).expect("Should be able to git init");
    }

    pub fn add_all(&self) {
        self.git_command(&["add", "-A"])
            .expect("Should be able to able to git add all files");
    }

    pub fn commit(&self, message: &str) -> String {
        self.git_command(&[
            "commit",
            "--author",
            GIT_COMMIT_AUTHOR,
            "--no-gpg-sign",
            "--message",
            message.trim(),
        ])
        .expect("Should be able to git commit");

        self.git_command(&["rev-parse", "HEAD"])
            .expect("Should be able to get last git commit id")
    }

    pub fn is_something_modified(&self) -> bool {
        let output = self
            .git_command(&["status", "--short"])
            .expect("Should be able to check git repository for modified status");
        !output.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn simple_command<P: AsRef<Path>>(dir: P, command: &str) -> anyhow::Result<String> {
        let parts = command.split(' ').collect::<Vec<_>>();
        let program = parts[0];
        let args = &parts[1..];
        let output = Command::new(program).args(args).current_dir(dir).output()?;
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            Ok(stdout.trim().to_owned())
        } else {
            Err(anyhow::anyhow!(
                "failed command \"{command}\" with {}\nStdout: {}\nStderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }

    fn init_test_env() -> anyhow::Result<(tempfile::TempDir, Repo)> {
        let tempdir = tempfile::Builder::new()
            .prefix("website-stalker-testing-")
            .tempdir()?;
        let dir = tempdir.path();

        let repo = Repo {
            dir: dir.to_path_buf(),
        };
        simple_command(dir, "git init")?;
        simple_command(dir, "git config user.email bla@blubb.de")?;
        simple_command(dir, "git config user.name Bla")?;
        Ok((tempdir, repo))
    }

    fn println_command<P: AsRef<Path>>(dir: P, command: &str) {
        println!("# {command}");
        match simple_command(dir, command) {
            Ok(output) => println!("{output}"),
            Err(err) => println!("{err}"),
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
        Repo::init(dir);
        assert_eq!(simple_command(dir, "git status --short")?, "");
        fs::write(dir.join("bla.txt"), "stuff")?;
        assert_eq!(simple_command(dir, "git status --short")?, "?? bla.txt");
        Ok(())
    }

    #[test]
    fn add_all_works() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        fs::write(dir.join("bla.txt"), "stuff")?;
        assert_eq!(simple_command(dir, "git status --short")?, "?? bla.txt");
        repo.add_all();
        assert_eq!(simple_command(dir, "git status --short")?, "A  bla.txt");
        Ok(())
    }

    #[test]
    fn commit_commits_with_existing_commits() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        simple_command(dir, "git commit -m init --allow-empty")?;
        fs::write(dir.join("bla.txt"), "stuff")?;
        simple_command(dir, "git add bla.txt")?;
        overview(dir);
        assert_eq!(simple_command(dir, "git log")?.lines().count(), 5);
        repo.commit("bla");
        overview(dir);
        assert_eq!(simple_command(dir, "git log")?.lines().count(), 11);
        Ok(())
    }

    #[test]
    fn commit_commits_empty_repo() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        fs::write(dir.join("bla.txt"), "stuff")?;
        simple_command(dir, "git add bla.txt")?;
        overview(dir);
        repo.commit("bla");
        overview(dir);
        assert_eq!(simple_command(dir, "git log")?.lines().count(), 5);
        Ok(())
    }

    #[test]
    fn is_something_modified_untracked() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        assert!(!repo.is_something_modified());

        fs::write(dir.join("bla.txt"), "stuff")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git add bla.txt")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git reset")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git clean -xdf")?;
        assert!(!repo.is_something_modified());
        Ok(())
    }

    #[test]
    fn is_something_modified_changed() -> anyhow::Result<()> {
        let (tempdir, repo) = init_test_env()?;
        let dir = tempdir.path();
        assert!(!repo.is_something_modified());

        fs::write(dir.join("bla.txt"), "foo")?;
        simple_command(dir, "git add bla.txt")?;
        simple_command(dir, "git commit -m bla")?;
        assert!(!repo.is_something_modified());

        fs::write(dir.join("bla.txt"), "bar")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git add bla.txt")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git reset")?;
        assert!(repo.is_something_modified());
        simple_command(dir, "git checkout bla.txt")?;
        assert!(!repo.is_something_modified());
        Ok(())
    }
}
