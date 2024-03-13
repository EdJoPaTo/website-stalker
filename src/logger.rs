use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::Lazy;

static GHA: Lazy<bool> = Lazy::new(|| env::var_os("GITHUB_ACTIONS").is_some());

pub fn error_exit(message: &str) -> ! {
    error(message);
    std::process::exit(1);
}

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

/// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-output-parameter>
/// <https://docs.github.com/en/actions/learn-github-actions/contexts#steps-context>
pub fn gha_output(key: &str, text: &str) {
    fn inner(key: &str, text: &str) -> std::io::Result<()> {
        static GITHUB_OUTPUT: Lazy<Option<PathBuf>> =
            Lazy::new(|| env::var_os("GITHUB_OUTPUT").map(PathBuf::from));
        if let Some(output) = &*GITHUB_OUTPUT {
            let mut file = OpenOptions::new().create(true).append(true).open(output)?;
            // <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#multiline-strings>
            writeln!(file, "{key}<<EOF\n{text}\nEOF")?;
        }
        Ok(())
    }
    inner(key, text).expect("should be able to write to GITHUB_OUTPUT");
}

pub fn warn_deprecated_notifications() {
    static HAS_WARNED: AtomicBool = AtomicBool::new(false);
    let before = HAS_WARNED.swap(true, Ordering::Relaxed);
    if !before {
        warn("Notifications are deprecated and will be replaced by a simpler machine-readable output. This way you will be able to control the exact notifications even better yourself. The details on this are not yet finalized. Please join the discussion on https://github.com/EdJoPaTo/website-stalker/discussions/172");
    }
}
