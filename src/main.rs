use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::Duration;
use std::{fs, process};

use crate::config::Config;
use crate::site::Site;
use crate::site_store::SiteStore;
use itertools::Itertools;
use regex::Regex;
use tokio::sync::RwLock;
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
    let matches = cli::build().get_matches();
    match matches.subcommand().expect("expected a subcommand") {
        ("example-config", _) => {
            println!(
                "# This is an example config
# The filename should be `website-stalker.yaml`
# and it should be in the working directory where you run website-stalker.
#
# For example run `website-stalker example-config > website-stalker.yaml`.
# Adapt the config to your needs and set the FROM email address which is used as a request header:
# https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From
#
# And then do a run via `website-stalker run --all`.
{}",
                Config::example_yaml_string()
            );
        }
        ("init", _) => {
            if git::Repo::new().is_err() {
                git::Repo::init(std::env::current_dir().expect("failed to get working dir path"))
                    .expect("failed to init repo");
                println!("Git repo initialized.");
            }
            if Config::load().is_err() {
                let contents = format!(
                    "# This is an example config
# Adapt it to your needs and check if its valid via `website-stalker check`.
# In order to run use `website-stalker run --all`.
{}",
                    Config::example_yaml_string()
                );
                fs::write("website-stalker.yaml", contents)
                    .expect("failed to write example config file");
                println!("Example config file generated.");
            }
            println!("Init complete.\nNext step: adapt the config file to your needs.");
        }
        ("check", matches) => {
            let notifiers = pling::Notifier::from_env().len();
            eprintln!("Notifiers: {}. Check https://github.com/EdJoPaTo/pling/ for configuration details.", notifiers);

            eprintln!("\nConfig...");
            match Config::load() {
                Ok(config) => {
                    let print_yaml = matches.is_present("print-yaml");
                    let rewrite_yaml = matches.is_present("rewrite-yaml");
                    if print_yaml || rewrite_yaml {
                        let yaml = serde_yaml::to_string(&config).expect("failed to parse to yaml");
                        if rewrite_yaml {
                            fs::write("website-stalker.yaml", &yaml)
                                .expect("failed to write website-stalker.yaml");
                        }
                        if print_yaml {
                            println!("{}", yaml);
                        }
                    }

                    eprintln!("ok");
                }
                Err(err) => {
                    eprintln!("not ok.\n\n{}\n\nCheck https://github.com/EdJoPaTo/website-stalker for configuration details.", err);
                    process::exit(1);
                }
            }
        }
        ("run", matches) => {
            let do_commit = matches.is_present("commit");
            let site_filter = matches
                .value_of("site filter")
                .map(|v| Regex::new(&format!("(?i){}", v)).unwrap());
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
        (subcommand, matches) => {
            todo!("subcommand {} {:?}", subcommand, matches);
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
    if let Ok(repo) = &repo {
        if repo.is_something_modified()? {
            if do_commit {
                return Err(anyhow::anyhow!(
                    "The repo is unclean. --commit can only be used in a clean repo."
                ));
            }
            logger::warn("The repo is unclean.");
        }
    } else {
        if do_commit {
            return Err(anyhow::anyhow!(
                "Not a git repo. --commit only works in git repos."
            ));
        }
        logger::warn("Not a git repo. Will run but won't do git actions.");
    }

    let site_store = site_store::SiteStore::new(SITE_FOLDER.to_string())
        .expect("failed to create sites directory");

    if sites_amount == sites_total {
        let basenames = Site::get_all_file_basenames(&sites);
        let removed = site_store.remove_gone(&basenames)?;
        for filename in removed {
            logger::warn(&format!("Remove superfluous {:?}", filename));
        }
    }

    if sites_amount < sites_total {
        logger::info(&format!(
            "Your config contains {} sites of which {} are selected by your filter.",
            sites_total, sites_amount
        ));
    }
    println!(
        "Begin stalking of {} sites on {} domains...",
        sites_amount, distinct_domains
    );
    if distinct_domains < sites_amount {
        logger::info("Some sites are on the same domain. There is a wait time of 5 seconds between each request to the same domain in order to reduce load on the server.");
    }

    let mut tasks = Vec::with_capacity(sites_amount);
    let groups = sites
        .into_iter()
        .group_by(|a| a.url.domain().unwrap().to_string());
    let amount_done = Arc::new(RwLock::new(0_usize));
    for (_, group) in &groups {
        for (i, site) in group.enumerate() {
            let site_store = site_store.clone();
            let from = config.from.clone();
            let amount_done = amount_done.clone();
            let handle = tokio::spawn(async move {
                sleep(Duration::from_secs((i * 5) as u64)).await;
                let result = stalk_and_save_site(&site_store, &from, &site).await;
                let url = site.url.as_str();

                let mut done = amount_done.write().await;
                *done += 1;

                match result {
                    Ok((change_kind, ip_version, took)) => {
                        println!(
                            "{:4}/{} {:12} {:5}ms {} {}",
                            done,
                            sites_amount,
                            change_kind.to_string(),
                            took.as_millis(),
                            ip_version,
                            url
                        );
                        Ok((site, change_kind))
                    }
                    Err(err) => {
                        logger::error(&format!("{} {}", url, err));
                        Err(err)
                    }
                }
            });

            tasks.push(handle);
        }
    }

    let mut sites_of_interest = Vec::new();
    let mut error_occured = false;
    for handle in tasks {
        match handle.await.expect("failed to spawn task") {
            Ok((site, change_kind)) => match change_kind {
                ChangeKind::Init | ChangeKind::Changed => {
                    sites_of_interest.push(site);
                }
                ChangeKind::ContentSame => {}
            },
            Err(_) => {
                error_occured = true;
            }
        }
    }

    let message = final_message::FinalMessage::new(&sites_of_interest);
    let commit = if let Ok(repo) = repo {
        run_commit(&repo, do_commit, &message.to_commit())?
    } else {
        None
    };
    if !sites_of_interest.is_empty() {
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
    let response = http::get(site.url.as_str(), from, site.options.accept_invalid_certs).await?;
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
            logger::error(&format!(
                "notifier failed to send with Err: {}\n{:?}",
                err, notifier,
            ));
        }
    }
}
