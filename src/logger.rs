use std::env;
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

pub fn warn_deprecated_notifications() {
    static HAS_WARNED: AtomicBool = AtomicBool::new(false);
    let before = HAS_WARNED.swap(true, Ordering::Relaxed);
    if !before {
        warn("Notifications are deprecated and will be replaced by a simpler machine-readable output. This way you will be able to control the exact notifications even better yourself. The details on this are not yet finalized. Please join the discussion on https://github.com/EdJoPaTo/website-stalker/discussions/172");
    }
}
