use http::Http;
use settings::Settings;
use site::Site;

use crate::site::Huntable;

mod cli;
mod git;
mod http;
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
            let settings = Settings::load().expect("failed to load settings");
            std::fs::create_dir_all("sites").expect("failed to create sites directory");
            let mut http_agent = http::Http::new(settings.from);
            if let Some(user_agent) = settings.user_agent {
                http_agent.set_user_agent(user_agent);
            }

            let is_repo = git::is_repo();
            if is_repo {
                git::reset().unwrap();
            } else {
                println!("HINT: not a git repo. Will run but won't do git actions.")
            }

            println!("Begin stalking sites...\n");

            for site in settings.sites {
                if let Err(err) = do_site(&http_agent, is_repo, &site) {
                    println!("  site failed: {}", err);
                }

                println!();
            }

            if matches.is_present("commit") {
                git::commit("website stalker stalked some things").unwrap();
            } else if is_repo {
                git::diff(&["--staged", "--stat"]).unwrap();
            }
            if is_repo {
                git::status_short().unwrap();
            }

            println!("\nAll done. Thanks for using website-stalker!");
        }
        (subcommand, matches) => {
            todo!("subcommand {} {:?}", subcommand, matches);
        }
    }
}

fn do_site(http_agent: &Http, is_repo: bool, site: &Site) -> anyhow::Result<()> {
    println!("do site {:?}", site);

    let filename = site.get_filename();
    let contents = site.hunt(http_agent)?;
    let contents = contents.trim().to_string() + "\n";
    println!("  filename {}", filename);
    println!("  content length {}", contents.len());

    let path = format!("sites/{}", filename);
    std::fs::write(&path, contents)?;

    if is_repo {
        git::add(&path)?;
    }

    Ok(())
}
