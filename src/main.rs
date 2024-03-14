use core::time::Duration;
use std::collections::HashMap;
use std::{fs, process};

use clap::Parser;
use regex::Regex;
use reqwest::header::{HeaderValue, FROM};
use tokio::sync::mpsc::channel;
use tokio::time::sleep;

use crate::cli::Cli;
use crate::config::{Config, EXAMPLE_CONF};
use crate::site::Site;

mod cli;
mod commit_message;
mod config;
mod editor;
mod filename;
mod git;
mod http;
mod logger;
mod notification;
mod site;
mod site_store;

pub enum ChangeKind {
    Init,
    Changed,
    ContentSame,
}

impl core::fmt::Display for ChangeKind {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Init => fmt.pad("Init"),
            Self::Changed => fmt.pad("Changed"),
            Self::ContentSame => fmt.pad("ContentSame"),
        }
    }
}

#[tokio::main]
async fn main() {
    match Cli::parse() {
        Cli::ExampleConfig => print!("{EXAMPLE_CONF}"),
        Cli::Init => {
            if git::Repo::new().is_err() {
                git::Repo::init(
                    &std::env::current_dir().expect("Should be run in a valid working directory"),
                );
                println!("Git repository initialized.");
            }
            if Config::load().is_err() {
                fs::write("website-stalker.yaml", EXAMPLE_CONF)
                    .expect("failed to write example configuration file");
                println!("Example configuration file generated.");
            }
            println!("Init complete.\nNext step: adapt the configuration file to your needs.");
        }
        Cli::Check => {
            let notifiers = pling::Notifier::from_env().len();
            eprintln!("Notifiers: {notifiers}. Check https://github.com/EdJoPaTo/pling/ for configuration details.");

            eprintln!("\nConfiguration...");
            match Config::load() {
                Ok(_) => eprintln!("ok"),
                Err(err) => {
                    eprintln!("not ok.\n\n{err}\n\nCheck https://github.com/EdJoPaTo/website-stalker for configuration details.");
                    process::exit(1);
                }
            }
        }
        Cli::Run {
            commit: do_commit,
            site_filter,
            ..
        } => {
            let site_filter =
                site_filter.map(|regex| Regex::new(&format!("(?i){}", regex.as_str())).unwrap());
            run(do_commit, site_filter.as_ref()).await;
            eprintln!("Thank you for using website-stalker!");
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn run(do_commit: bool, site_filter: Option<&Regex>) {
    let config = Config::load().expect("failed to load your configuration");
    let from = config
        .from
        .parse::<HeaderValue>()
        .expect("FROM has to be valid");

    let sites = config.get_sites();
    let sites_total = sites.len();
    let sites = sites
        .into_iter()
        .filter(|site| site_filter.map_or(true, |filter| filter.is_match(site.url.as_str())))
        .collect::<Vec<_>>();
    let sites_amount = sites.len();
    if sites.is_empty() {
        logger::error_exit(
            "The site-filter filtered everything out. Change the filter or use all sites with 'run --all'.",
        );
    }

    let repo = git::Repo::new();
    match &repo {
        Ok(repo) => {
            if repo.is_something_modified() {
                if do_commit {
                    logger::error_exit("The git repository is unclean. --commit can only be used in a clean repository.");
                }
                logger::warn("The git repository is unclean.");
            }
        }
        Err(err) => {
            if do_commit {
                logger::error_exit(&format!(
                    "Not a git repository. --commit only works in git repos: {err}"
                ));
            }
            logger::warn("Not a git repository. Will run but won't do git actions.");
        }
    }

    if sites_amount == sites_total {
        let paths = Site::get_all_file_paths(&sites);
        let removed = site_store::remove_gone(&paths)
            .expect("Should be able to cleanup the superfluous files");
        for filename in removed {
            logger::warn(&format!("Remove superfluous {filename:?}"));
        }
    }

    if sites_amount < sites_total {
        logger::info(&format!("Your configuration file contains {sites_total} sites of which {sites_amount} are selected by your filter."));
    }

    let mut groups: HashMap<String, Vec<Site>> = HashMap::new();
    for site in sites {
        let host = site.url.host_str().unwrap().to_owned();
        groups.entry(host).or_default().push(site);
    }

    let distinct_hosts = groups.len();
    println!("Begin stalking of {sites_amount} sites on {distinct_hosts} hosts...");
    if distinct_hosts < sites_amount {
        logger::info("Some sites are on the same host. There is a wait time of 5 seconds between each request to the same host in order to reduce load on the server.");
    }

    let mut rx = {
        let (tx, rx) = channel(10);
        for (_, sites) in groups {
            let from = from.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                for (i, site) in sites.into_iter().enumerate() {
                    if i > 0 {
                        sleep(Duration::from_secs(5)).await;
                    }
                    let result = stalk_and_save_site(&from, &site).await;
                    tx.send((site.url, result, site.options.ignore_error))
                        .await
                        .expect("failed to send stalking result");
                }
            });
        }
        rx
    };

    let mut urls_of_interest = Vec::new();
    let mut error_occured = false;
    let mut amount_done: usize = 0;
    while let Some((url, result, ignore_error)) = rx.recv().await {
        amount_done += 1;
        match result {
            Ok((change_kind, ip_version, took)) => {
                println!(
                    "{amount_done:4}/{sites_amount} {change_kind:11} {:5}ms {ip_version} {url}",
                    took.as_millis(),
                );
                match change_kind {
                    ChangeKind::Init | ChangeKind::Changed => {
                        urls_of_interest.push(url);
                    }
                    ChangeKind::ContentSame => {}
                }
            }
            Err(err) => {
                let message = format!("{url} {err}");
                if ignore_error {
                    logger::warn(&message);
                } else {
                    logger::error(&message);
                    error_occured = true;
                }
            }
        }
    }

    let commit = repo
        .ok()
        .filter(git::Repo::is_something_modified)
        .and_then(|repo| {
            if do_commit {
                repo.add_all();
                let message = commit_message::commit_message(&urls_of_interest);
                let id = repo.commit(&message);
                Some(id)
            } else {
                logger::warn("No commit is created without the --commit flag.");
                None
            }
        });

    if !urls_of_interest.is_empty() {
        let notifiers = pling::Notifier::from_env();
        if !notifiers.is_empty() {
            let message = notification::MustacheData::new(commit, urls_of_interest)
                .apply_to_template(config.notification_template.as_ref())
                .expect("Should be able to create notification message from template");
            for notifier in notifiers {
                if let Err(err) = notifier.send_sync(&message) {
                    logger::error(&format!("notifier failed to send with Err: {err}"));
                }
            }
        }
    }

    if error_occured {
        logger::error_exit("All done but some site failed. Thank you for using website stalker!");
    }
}

async fn stalk_and_save_site(
    from: &HeaderValue,
    site: &Site,
) -> anyhow::Result<(ChangeKind, http::IpVersion, Duration)> {
    let mut headers = site.options.headers.clone();
    if !headers.contains_key(FROM) {
        headers.insert(FROM, from.clone());
    }
    let response = http::get(
        site.url.as_str(),
        headers,
        site.options.accept_invalid_certs,
    )
    .await?;
    let took = response.took();
    let ip_version = response.ip_version();

    if site.url.as_str() != response.url().as_str() {
        logger::warn(&format!("The URL {} was redirected to {}. This caused additional traffic which can be reduced by changing the URL to the target one.", site.url, response.url()));
    }

    let url = response.url().clone();
    let content = editor::Content {
        extension: response.file_extension(),
        text: response.text().await?,
    };

    // Use response.url as canonical urls for example are relative to the actual url
    let content = editor::Editor::apply_many(&site.options.editors, &url, content)?;
    let extension = content.extension.unwrap_or("txt");

    // Use site.url as the file basename should only change when the config changes (manually)
    let mut path = site.to_file_path();
    path.set_extension(extension);
    let changed = site_store::write_only_changed(&path, &content.text)?;
    Ok((changed, ip_version, took))
}
