use std::sync::Arc;
use std::time::{Duration, Instant};

use http::Http;
use itertools::Itertools;
use regex::Regex;
use settings::Settings;
use site::Site;
use tokio::sync::RwLock;
use tokio::time::sleep;

mod cli;
mod git;
mod http;
mod logger;
mod regex_replacer;
mod settings;
mod site;

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
            match run(do_commit, site_filter.as_ref()).await {
                Ok(_) => {
                    println!("\nAll done. Thanks for using website-stalker!");
                }
                Err(err) => {
                    println!("\n{} Thanks for using website-stalker!", err);
                    std::process::exit(1);
                }
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
        git::cleanup("sites").unwrap();
    } else {
        logger::warn("Not a git repo. Will run but won't do git actions.");
    }

    std::fs::create_dir_all("sites").expect("failed to create sites directory");

    let filenames = sites.iter().map(Site::get_filename).collect::<Vec<_>>();
    let something_removed = if sites_amount == sites_total {
        remove_gone_sites(&filenames)?
    } else {
        false
    };

    if sites_amount < sites_total {
        println!(
            "Begin filtered stalking of {}/{} sites on {} domains...",
            sites_amount, sites_total, distinct_domains
        );
    } else {
        println!(
            "Begin stalking {} sites on {} domains...",
            sites_amount, distinct_domains
        );
    }
    if distinct_domains < sites_amount {
        logger::hint("Some sites are on the same domain. There is a wait time of 5 seconds between each request to the same domain in order to reduce load on the server.");
    }

    let mut tasks = Vec::with_capacity(sites_amount);
    let groups = sites
        .into_iter()
        .group_by(|a| a.get_url().domain().unwrap().to_string());
    let amount_done = Arc::new(RwLock::new(0_usize));
    for (_, group) in &groups {
        for (i, site) in group.enumerate() {
            let http_agent = http_agent.clone();
            let amount_done = amount_done.clone();
            let handle = tokio::spawn(async move {
                sleep(Duration::from_secs((i * 5) as u64)).await;
                let result = stalk_and_save_site(&http_agent, &site).await;
                let url = site.get_url().as_str();

                let mut done = amount_done.write().await;
                *done += 1;

                match &result {
                    Ok((changed, took)) => {
                        let change = if *changed { "  CHANGED" } else { "UNCHANGED" };
                        println!(
                            "{:4}/{} {} {:5}ms {}",
                            done,
                            sites_amount,
                            change,
                            took.as_millis(),
                            url
                        );
                    }
                    Err(err) => {
                        logger::error(&format!("{} {}", url, err));
                    }
                }
                result
            });

            tasks.push(handle);
        }
    }

    let mut something_changed = false;
    let mut error_occured = false;
    for handle in tasks {
        match handle.await.expect("failed to spawn task") {
            Ok((true, _)) => {
                something_changed = true;
            }
            Ok(_) => {}
            Err(_) => {
                error_occured = true;
            }
        }
    }

    if is_repo {
        println!();
        if something_removed || something_changed {
            git_finishup(do_commit)?;
        }
        git::status_short()?;
    }

    if error_occured {
        Err(anyhow::anyhow!("All done but some site failed."))
    } else {
        Ok(())
    }
}

async fn stalk_and_save_site(http_agent: &Http, site: &Site) -> anyhow::Result<(bool, Duration)> {
    let path = format!("sites/{}", site.get_filename());
    let start = Instant::now();
    let response = http_agent.get(site.get_url().as_str()).await?;
    let took = Instant::now().saturating_duration_since(start);
    let contents = site.stalk(response).await?;
    let contents = contents.trim().to_string() + "\n";
    let changed = write_only_changed(path, &contents)?;
    Ok((changed, took))
}

fn write_only_changed<P>(path: P, contents: &str) -> std::io::Result<bool>
where
    P: AsRef<std::path::Path>,
{
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    let changed = current != contents;
    if changed {
        std::fs::write(&path, contents)?;
    }
    Ok(changed)
}

fn remove_gone_sites(existing_filenames: &[String]) -> anyhow::Result<bool> {
    let mut any_removed = false;

    for file in std::fs::read_dir("sites")? {
        let file = file?;
        let name = file
            .file_name()
            .into_string()
            .map_err(|name| anyhow::anyhow!("filename has no valid Utf-8: {:?}", name))?;

        let is_wanted = existing_filenames.as_ref().contains(&name);
        if !is_wanted {
            let path = format!("sites/{}", name);
            std::fs::remove_file(&path)?;
            any_removed = true;
            logger::warn(&format!("Remove superfluous {}", path));
        }
    }

    Ok(any_removed)
}

fn git_finishup(do_commit: bool) -> anyhow::Result<()> {
    git::add(&["sites"])?;
    git::diff(&["--staged", "--stat"])?;

    if do_commit {
        logger::begin_group("git commit");
        git::commit("stalked some things \u{1f440}\u{1f310}\u{1f60e}")?;
        logger::end_group();
    } else {
        logger::warn("No commit is created without the --commit flag.");
    }

    Ok(())
}
