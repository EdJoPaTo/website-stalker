use http::Http;
use regex::Regex;
use settings::{Settings, Site};

mod cli;
mod git;
mod http;
mod settings;

fn main() {
    let matches = cli::build().get_matches();
    match matches.subcommand() {
        ("example-config", _) => {
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
        ("check", _) => {
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
        ("run", _) => {
            let settings = Settings::load().expect("failed to load settings");
            std::fs::create_dir_all("sites").expect("failed to create sites directory");
            let mut http_agent = http::Http::new(settings.from);
            if let Some(user_agent) = settings.user_agent {
                http_agent.set_user_agent(user_agent);
            }

            let is_repo = git::is_repo();
            if is_repo {
                git::reset().expect("failed to reset git repo state");
            } else {
                println!("HINT: not a git repo. Will run but wont commit.")
            }

            println!("Begin stalking sites...\n");

            for site in settings.sites {
                if let Err(err) = do_site(&http_agent, is_repo, &site) {
                    println!("  site failed: {}", err);
                }

                println!();
            }

            if is_repo {
                drop(git::commit("website stalker stalked some things"));
            }

            println!("All done. Thanks for using website-stalker!");
        }
        (subcommand, matches) => {
            todo!("subcommand {} {:?}", subcommand, matches);
        }
    }
}

fn do_site(http_agent: &Http, is_repo: bool, site: &Site) -> anyhow::Result<()> {
    println!("do site {:?}", site);

    let url = match site {
        Site::Html(http) => &http.url,
        Site::Utf8(utf8) => &utf8.url,
    };
    let text = http_agent.get(url.as_str())?;
    println!("  text length {}", text.len());

    let filename = format!("sites/{}.html", format_url_as_filename(&url));
    std::fs::write(&filename, text)?;
    if is_repo {
        git::add(&filename)?;
    }

    Ok(())
}

fn format_url_as_filename(url: &url::Url) -> String {
    let re = Regex::new("[^a-zA-Z\\d]+").unwrap();
    let bla = re.replace_all(url.as_str(), "-");
    let blubb = bla.trim_matches('-');
    blubb.to_string()
}
