use std::fmt::{Debug, Display};
use std::time::Duration;
use std::{fs, process};

use crate::cli::SubCommand;
use crate::config::Config;
use crate::site::Site;
use crate::site_store::SiteStore;
use clap::Parser;
use itertools::Itertools;
use regex::Regex;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;

mod cli;
mod config;
mod editor;
mod filename;
mod final_message;
mod git;
mod http;
mod logger;
mod site;
mod site_store;

const SITE_FOLDER: &str = "sites";

#[derive(Debug)]
pub enum ChangeKind {
    Init,
    Changed,
    ContentSame,
}

impl Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[tokio::main]
async fn main() {
    match cli::Cli::parse().subcommand {
        SubCommand::ExampleConfig => {
            println!(
                "{}{}",
                config::EXAMPLE_CONF,
                Config::example_yaml_string()
            );
        }
        SubCommand::Init => {
            if git::Repo::new().is_err() {
                git::Repo::init(std::env::current_dir().expect("failed to get working dir path"))
                    .expect("failed to init repo");
                println!("Git repo initialized.");
            }
            if Config::load().is_err() {
                let contents = format!(
                    "{}{}",
                    config::EXAMPLE_CONF,
                    Config::example_yaml_string()
                );
                fs::write("website-stalker.yaml", contents)
                    .expect("failed to write example config file");
                println!("Example config file generated.");
            }
            println!("Init complete.\nNext step: adapt the config file to your needs.");
        }
        SubCommand::Check {
            print_yaml,
            rewrite_yaml,
        } => {
            let notifiers = pling::Notifier::from_env().len();
            eprintln!("Notifiers: {notifiers}. Check https://github.com/EdJoPaTo/pling/ for configuration details.");

            eprintln!("\nConfig...");
            match Config::load() {
                Ok(config) => {
                    if print_yaml || rewrite_yaml {
                        let yaml = serde_yaml::to_string(&config).expect("failed to parse to yaml");
                        if rewrite_yaml {
                            fs::write("website-stalker.yaml", &yaml)
                                .expect("failed to write website-stalker.yaml");
                        }
                        if print_yaml {
                            println!("{yaml}");
                        }
                    }

                    eprintln!("ok");
                }
                Err(err) => {
                    eprintln!("not ok.\n\n{err}\n\nCheck https://github.com/EdJoPaTo/website-stalker for configuration details.");
                    process::exit(1);
                }
            }
        }
        SubCommand::Run {
            commit: do_commit,
            site_filter,
            ..
        } => {
            let site_filter =
                site_filter.map(|v| Regex::new(&format!("(?i){}", v.as_str())).unwrap());
            let result = run(do_commit, site_filter.as_ref()).await;
            if let Err(err) = &result {
                logger::error(&err.to_string());
            } else {
                println!("All done.");
            }
            println!("Thanks for using website-stalker!");
            if result.is_err() {
                process::exit(1);
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn run(do_commit: bool, site_filter: Option<&Regex>) -> anyhow::Result<()> {
    let config = Config::load().expect("failed to load config");

    let sites = config.get_sites();
    let sites_total = sites.len();
    let sites = sites
        .into_iter()
        .filter(|site| site_filter.map_or(true, |filter| filter.is_match(site.url.as_str())))
        .collect::<Vec<_>>();
    let sites_amount = sites.len();
    assert!(!sites.is_empty(), "Site filter filtered everything out.");

    let distinct_domains = {
        let mut domains = sites
            .iter()
            .map(|o| o.url.domain().unwrap().to_string())
            .collect::<Vec<_>>();
        domains.sort();
        domains.dedup();
        domains.len()
    };

    let repo = git::Repo::new();
    match &repo {
        Ok(repo) => {
            if repo.is_something_modified()? {
                if do_commit {
                    anyhow::bail!(
                        "The repo is unclean. --commit can only be used in a clean repo."
                    );
                }
                logger::warn("The repo is unclean.");
            }
        }
        Err(err) => {
            if do_commit {
                anyhow::bail!("Not a git repo. --commit only works in git repos: {err}");
            }
            logger::warn("Not a git repo. Will run but won't do git actions.");
        }
    }

    let site_store = site_store::SiteStore::new(SITE_FOLDER.to_string())
        .expect("failed to create sites directory");

    if sites_amount == sites_total {
        let basenames = Site::get_all_file_basenames(&sites);
        let removed = site_store.remove_gone(&basenames)?;
        for filename in removed {
            logger::warn(&format!("Remove superfluous {filename:?}"));
        }
    }

    if sites_amount < sites_total {
        logger::info(&format!("Your config contains {sites_total} sites of which {sites_amount} are selected by your filter."));
    }
    println!("Begin stalking of {sites_amount} sites on {distinct_domains} domains...");
    if distinct_domains < sites_amount {
        logger::info("Some sites are on the same domain. There is a wait time of 5 seconds between each request to the same domain in order to reduce load on the server.");
    }

    let mut rx = {
        let (tx, rx) = channel(10);
        let groups = sites
            .into_iter()
            .group_by(|a| a.url.domain().unwrap().to_string());
        for (_, group) in &groups {
            let from = config.from.clone();
            let site_store = site_store.clone();
            let sites = group.collect::<Vec<_>>();
            let tx = tx.clone();
            tokio::spawn(async move {
                for (i, site) in sites.iter().enumerate() {
                    if i > 0 {
                        sleep(Duration::from_secs(5)).await;
                    }
                    let result = stalk_and_save_site(&site_store, &from, site).await;
                    tx.send((site.url.clone(), result, site.options.ignore_error))
                        .await
                        .expect("failed to send stalking result");
                }
            });
        }
        rx
    };

    let mut urls_of_interest = Vec::new();
    let mut error_occured = false;
    let mut amount_done = 0_usize;
    while let Some((url, result, ignore_error)) = rx.recv().await {
        amount_done += 1;
        match result {
            #[allow(clippy::to_string_in_format_args)]
            Ok((change_kind, ip_version, took)) => {
                println!(
                    "{amount_done:4}/{sites_amount} {:12} {:5}ms {ip_version} {url}",
                    change_kind.to_string(),
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

    let message = final_message::FinalMessage::new(&urls_of_interest);
    let commit = if let Ok(repo) = repo {
        run_commit(&repo, do_commit, &message.to_commit())?
    } else {
        None
    };
    if !urls_of_interest.is_empty() {
        let message = message.into_notification(config.notification_template.as_deref(), commit)?;
        run_notifications(&message);
    }

    if error_occured {
        Err(anyhow::anyhow!("All done but some site failed."))
    } else {
        Ok(())
    }
}

async fn stalk_and_save_site(
    site_store: &SiteStore,
    from: &str,
    site: &Site,
) -> anyhow::Result<(ChangeKind, http::IpVersion, Duration)> {
    let response = http::get(
        site.url.as_str(),
        &site.options.headers,
        from,
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
    let content = editor::apply_many(&site.options.editors, &url, content)?;
    let extension = content.extension.unwrap_or("txt");

    // Use site.url as the file basename should only change when the config changes (manually)
    let basename = filename::basename(&site.url);
    let changed = site_store.write_only_changed(&basename, extension, &content.text)?;
    Ok((changed, ip_version, took))
}

fn run_commit(repo: &git::Repo, do_commit: bool, message: &str) -> anyhow::Result<Option<String>> {
    if repo.is_something_modified()? {
        if do_commit {
            repo.add_all()?;
            let id = repo.commit(message)?;
            Ok(Some(id.to_string()))
        } else {
            logger::warn("No commit is created without the --commit flag.");
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn run_notifications(message: &str) {
    for notifier in pling::Notifier::from_env() {
        if let Err(err) = notifier.send_sync(message) {
            logger::error(&format!("notifier failed to send with Err: {err}"));
        }
    }
}
