use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fs, process};

use config::Config;
use http::Http;
use itertools::Itertools;
use regex::Regex;
use site::Site;
use site_store::SiteStore;
use tokio::sync::RwLock;
use tokio::time::sleep;

mod cli;
mod config;
mod editor;
mod git;
mod http;
mod logger;
mod serde_helper;
mod site;
mod site_store;
mod url_filename;

const SITE_FOLDER: &str = "sites";

#[derive(Debug)]
pub enum ChangeKind {
    Init,
    Changed,
    ContentSame,
    NotModified,
}

impl Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[tokio::main]
async fn main() {
    let matches = cli::build().get_matches();
    match matches.subcommand() {
        ("example-config", Some(_)) => {
            println!(
                "# This is an example config
# The filename should be `website-stalker.yaml`
# and it should be in the working directory where you run website-stalker.
#
# For example run `website-stalker example-config > website-stalker.yaml`.
# And then do a run via `website-stalker run --all`.
{}",
                Config::example_yaml_string()
            );
        }
        ("init", Some(_)) => {
            if git::Repo::new().is_err() {
                git::Repo::init(std::env::current_dir().expect("failed to get working dir path"))
                    .expect("failed to init repo");
                println!("Git repo initialized.");
            }
            if Config::load_yaml_file().is_err() {
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
        ("check", Some(_)) => match Config::load_yaml_file() {
            Ok(_) => println!("config ok"),
            Err(err) => {
                eprintln!("{}\n\nconfig not ok", err);
                process::exit(1);
            }
        },
        ("run", Some(matches)) => {
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
    let config = Config::load_yaml_file().expect("failed to load config");
    let http_agent = http::Http::new(&config.from);

    let sites_total = config.sites.len();
    let sites = config
        .sites
        .into_iter()
        .filter(|site| site_filter.map_or(true, |filter| filter.is_match(site.url.as_str())))
        .collect::<Vec<_>>();
    let sites_amount = sites.len();
    if sites.is_empty() {
        panic!("Site filter filtered everything out.");
    }

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
        let filenames = sites.iter().map(Site::get_filename).collect::<Vec<_>>();
        let removed = site_store.remove_gone(&filenames)?;
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
            let http_agent = http_agent.clone();
            let amount_done = amount_done.clone();
            let handle = tokio::spawn(async move {
                sleep(Duration::from_secs((i * 5) as u64)).await;
                let result = stalk_and_save_site(&site_store, &http_agent, &site).await;
                let url = site.url.as_str();

                let mut done = amount_done.write().await;
                *done += 1;

                match result {
                    Ok((change_kind, took)) => {
                        println!(
                            "{:4}/{} {:12} {:5}ms {}",
                            done,
                            sites_amount,
                            change_kind.to_string(),
                            took.as_millis(),
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
                    sites_of_interest.push((change_kind, site));
                }
                ChangeKind::ContentSame | ChangeKind::NotModified => {}
            },
            Err(_) => {
                error_occured = true;
            }
        }
    }

    if let Ok(repo) = &repo {
        git_finishup(repo, do_commit, &sites_of_interest)?;
    }

    if error_occured {
        Err(anyhow::anyhow!("All done but some site failed."))
    } else {
        Ok(())
    }
}

async fn stalk_and_save_site(
    site_store: &SiteStore,
    http_agent: &Http,
    site: &Site,
) -> anyhow::Result<(ChangeKind, Duration)> {
    let filename = site.get_filename();
    // TODO: get last known etag
    let etag = None;
    let start = Instant::now();
    let response = http_agent.get(site.url.as_str(), etag).await?;
    let took = Instant::now().saturating_duration_since(start);

    if site.url.as_str() != response.url().as_str() {
        logger::warn(&format!("The URL {} was redirected to {}. This caused additional traffic which can be reduced by changing the URL to the target one.", site.url, response.url()));
    }

    let changed = if response.is_not_modified() {
        ChangeKind::NotModified
    } else {
        let content = response.text().await?;
        let content = site.stalk(&content).await?;
        site_store.write_only_changed(&filename, &content)?
    };
    Ok((changed, took))
}

fn git_finishup(
    repo: &git::Repo,
    do_commit: bool,
    handled_sites: &[(ChangeKind, Site)],
) -> anyhow::Result<()> {
    if repo.is_something_modified()? {
        if do_commit {
            let message = if handled_sites.is_empty() {
                "just background magic \u{1f9fd}\u{1f52e}\u{1f9f9}\n\ncleanup or updating meta files"
                    .to_string() // 🧽🔮🧹
            } else {
                let mut lines = handled_sites
                    .iter()
                    .map(|(change_kind, site)| {
                        let letter = match change_kind {
                            ChangeKind::Init => 'A',
                            ChangeKind::Changed => 'M',
                            ChangeKind::ContentSame | ChangeKind::NotModified => unreachable!(),
                        };
                        format!("{} {}", letter, site.url.as_str())
                    })
                    .collect::<Vec<_>>();
                lines.sort();
                let body = lines.join("\n");
                format!(
                    "stalked some things \u{1f440}\u{1f310}\u{1f60e}\n\n{}", // 👀🌐😎
                    body
                )
            };
            repo.add_all()?;
            repo.commit(&message)?;
        } else {
            logger::warn("No commit is created without the --commit flag.");
        }
    }
    Ok(())
}
