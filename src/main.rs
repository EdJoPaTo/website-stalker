use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::{Duration, Instant};

use http::Http;
use itertools::Itertools;
use regex::Regex;
use settings::Settings;
use site::Site;
use site_store::SiteStore;
use tokio::sync::RwLock;
use tokio::time::sleep;

mod cli;
mod git;
mod http;
mod logger;
mod regex_replacer;
mod settings;
mod site;
mod site_store;

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
            let config = serde_yaml::to_string(&Settings::example()).unwrap();
            println!(
                "# This is an example config
# The filename should be `website-stalker.yaml`
# and it should be in the working directory where you run website-stalker.
#
# For example run `website-stalker example-config > website-stalker.yaml`.
# And then do a run via `website-stalker run`.
{}",
                config
            );
        }
        ("check", Some(_)) => {
            match Settings::load() {
                Ok(_) => println!("config ok"),
                Err(err) => {
                    eprintln!("{}", err);
                    // TODO: dont panic, just exit code != 0
                    eprintln!();
                    panic!("config not ok");
                }
            }
        }
        ("run", Some(matches)) => {
            let do_commit = matches.is_present("commit");
            let site_filter = matches
                .value_of("site filter")
                .map(|v| Regex::new(v).unwrap());
            let result = run(do_commit, site_filter.as_ref()).await;
            println!();
            if let Err(err) = &result {
                logger::error(&err.to_string());
            } else {
                println!("All done.");
            }
            println!("Thanks for using website-stalker!");
            if result.is_err() {
                std::process::exit(1);
            }
        }
        (subcommand, matches) => {
            todo!("subcommand {} {:?}", subcommand, matches);
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn run(do_commit: bool, site_filter: Option<&Regex>) -> anyhow::Result<()> {
    let settings = Settings::load().expect("failed to load settings");
    let http_agent = http::Http::new(&settings.from);

    let sites_total = settings.sites.len();
    let sites = settings
        .sites
        .into_iter()
        .filter(|site| site_filter.map_or(true, |filter| filter.is_match(site.get_url().as_str())))
        .collect::<Vec<_>>();
    let sites_amount = sites.len();
    if sites.is_empty() {
        panic!("Site filter filtered everything out.");
    }

    let distinct_domains = {
        let mut domains = sites
            .iter()
            .map(|o| o.get_url().domain().unwrap().to_string())
            .collect::<Vec<_>>();
        domains.sort();
        domains.dedup();
        domains.len()
    };

    let is_repo = git::is_repo();
    if is_repo {
        git::reset().unwrap();
        git::cleanup(SITE_FOLDER).unwrap();
    } else {
        logger::warn("Not a git repo. Will run but won't do git actions.");
    }

    let site_store = site_store::SiteStore::new(SITE_FOLDER.to_string())
        .expect("failed to create sites directory");

    let something_removed = if sites_amount == sites_total {
        let filenames = sites.iter().map(Site::get_filename).collect::<Vec<_>>();
        let removed = site_store.remove_gone(&filenames)?;
        for filename in &removed {
            logger::warn(&format!("Remove superfluous {:?}", filename));
        }
        !removed.is_empty()
    } else {
        false
    };

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
        .group_by(|a| a.get_url().domain().unwrap().to_string());
    let amount_done = Arc::new(RwLock::new(0_usize));
    for (_, group) in &groups {
        for (i, site) in group.enumerate() {
            let site_store = site_store.clone();
            let http_agent = http_agent.clone();
            let amount_done = amount_done.clone();
            let handle = tokio::spawn(async move {
                sleep(Duration::from_secs((i * 5) as u64)).await;
                let result = stalk_and_save_site(&site_store, &http_agent, &site).await;
                let url = site.get_url().as_str();

                let mut done = amount_done.write().await;
                *done += 1;

                match &result {
                    Ok((change_kind, took)) => {
                        println!(
                            "{:4}/{} {:12} {:5}ms {}",
                            done,
                            sites_amount,
                            change_kind.to_string(),
                            took.as_millis(),
                            url
                        );
                    }
                    Err(err) => {
                        logger::error(&format!("{} {}", url, err));
                    }
                }
                result.map(|(change_kind, took)| (site, change_kind, took))
            });

            tasks.push(handle);
        }
    }

    let mut sites_of_interest = Vec::new();
    let mut error_occured = false;
    for handle in tasks {
        match handle.await.expect("failed to spawn task") {
            Ok((site, change_kind, _)) => match change_kind {
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

    if is_repo {
        println!();
        if something_removed || !sites_of_interest.is_empty() {
            git_finishup(do_commit, &sites_of_interest)?;
        }
        git::status_short()?;
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
    let response = http_agent.get(site.get_url().as_str(), etag).await?;
    let took = Instant::now().saturating_duration_since(start);

    if response.is_not_modified() {
        return Ok((ChangeKind::NotModified, took));
    }

    let contents = site.stalk(response).await?;
    let changed = site_store.write_only_changed(&filename, &contents)?;
    Ok((changed, took))
}

fn git_finishup(do_commit: bool, handled_sites: &[(ChangeKind, Site)]) -> anyhow::Result<()> {
    git::add(&[SITE_FOLDER])?;
    git::diff(&["--staged", "--stat"])?;

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
                    format!("{} {}", letter, site.get_url().as_str())
                })
                .collect::<Vec<_>>();
            lines.sort();
            let body = lines.join("\n");
            format!(
                "stalked some things \u{1f440}\u{1f310}\u{1f60e}\n\n{}", // 👀🌐😎
                body
            )
        };
        git::commit(&message)?;
    } else {
        logger::warn("No commit is created without the --commit flag.");
    }

    Ok(())
}
