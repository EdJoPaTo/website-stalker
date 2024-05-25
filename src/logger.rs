use crate::github;

pub fn error_exit(message: &str) -> ! {
    error(message);
    std::process::exit(1);
}

pub fn error(message: &str) {
    if *github::IS_RUN_AS_GITHUB_ACTION {
        github::error(message);
    } else {
        eprintln!("ERROR: {message}");
    }
}

pub fn warn(message: &str) {
    if *github::IS_RUN_AS_GITHUB_ACTION {
        github::warning(message);
    } else {
        eprintln!("WARN: {message}");
    }
}

pub fn info(message: &str) {
    eprintln!("INFO: {message}");
}
