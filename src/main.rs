use http::Http;
use regex::Regex;
use settings::Settings;
use site::Site;

use crate::site::Huntable;

mod cli;
mod git;
mod http;
mod logger;
mod regex_replacer;
mod settings;
mod site;

fn main() {
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
            match run(do_commit, &site_filter) {
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

fn run(do_commit: bool, site_filter: &Option<Regex>) -> anyhow::Result<()> {
    let settings = Settings::load().expect("failed to load settings");
    let mut http_agent = http::Http::new(settings.from);
    if let Some(user_agent) = settings.user_agent {
        http_agent.set_user_agent(user_agent);
    }

    let sites_total = settings.sites.len();
    let sites = settings
        .sites
        .iter()
        .filter(|site| {
            site_filter
                .as_ref()
                .map_or(true, |filter| filter.is_match(site.get_url().as_str()))
        })
        .collect::<Vec<_>>();
    let sites_amount = sites.len();
    if sites.is_empty() {
        panic!("Site filter filtered everything out.");
    }

    let is_repo = git::is_repo();
    if is_repo {
        git::reset().unwrap();
        git::cleanup("sites").unwrap();
    } else {
        println!("HINT: not a git repo. Will run but won't do git actions.")
    }

    std::fs::create_dir_all("sites").expect("failed to create sites directory");

    if sites_amount == sites_total {
        let filenames = sites.iter().map(|o| o.get_filename()).collect::<Vec<_>>();
        remove_gone_sites(is_repo, do_commit, &filenames)?;
    }

    if sites_amount < sites_total {
        println!(
            "Begin filtered stalking of {}/{} sites...",
            sites_amount, sites_total
        );
    } else {
        println!("Begin stalking {} sites...", sites_amount);
    }

    let mut something_changed = false;
    let mut error_occured = false;

    for (i, site) in sites.iter().enumerate() {
        println!("{:4}/{} {}", i + 1, sites_amount, site.get_url().as_str());
        match do_site(&http_agent, is_repo, &site) {
            Ok(true) => {
                something_changed = true;
            }
            Ok(false) => {}
            Err(err) => {
                error_occured = true;
                logger::error(&err.to_string());
            }
        }
    }

    if is_repo {
        println!();
        git::diff(&["--staged", "--stat"]).unwrap();
    }
    if something_changed && do_commit {
        logger::begin_group("git commit");
        git::commit("stalked some things \u{1f440}\u{1f310}\u{1f60e}").unwrap();
        logger::end_group();
    }
    if is_repo {
        git::status_short().unwrap();
    }

    if error_occured {
        Err(anyhow::anyhow!("All done but some site failed."))
    } else {
        Ok(())
    }
}

fn do_site(http_agent: &Http, is_repo: bool, site: &Site) -> anyhow::Result<bool> {
    let filename = site.get_filename();
    let path = format!("sites/{}", filename);
    let contents = site.hunt(http_agent)?;
    let contents = contents.trim().to_string() + "\n";

    let current = std::fs::read_to_string(&path).unwrap_or_default();
    let changed = current != contents;
    if changed {
        std::fs::write(&path, contents)?;
    }
    if is_repo {
        // Always add as it could have changed in the last non --commit run
        git::add(&path)?;
    }

    Ok(changed)
}

fn remove_gone_sites(
    is_repo: bool,
    do_commit: bool,
    existing_filenames: &[String],
) -> anyhow::Result<()> {
    let mut any_removed = false;

    for file in std::fs::read_dir("sites")? {
        let file = file?;

        let name = file
            .file_name()
            .into_string()
            .map_err(|name| anyhow::anyhow!("filename has no valid Utf-8: {:?}", name))?;
        let path = format!("sites/{}", name);

        let is_wanted = existing_filenames.as_ref().contains(&name);
        if !is_wanted {
            std::fs::remove_file(&path)?;
            if is_repo {
                git::add(&path)?;
            }
            any_removed = true;

            logger::warn(&format!("remove superfluous {}", path));
        }
    }

    if any_removed && do_commit {
        logger::begin_group("git commit");
        git::commit("remove superfluous \u{1f5d1}\u{1f310}\u{1f916}")?;
        logger::end_group();
    }

    Ok(())
}
