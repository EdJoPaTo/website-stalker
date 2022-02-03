use clap::{app_from_crate, App, AppSettings, Arg};
use regex::Regex;

#[must_use]
pub fn build() -> App<'static> {
    app_from_crate!()
        .name("Website Stalker")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("example-config")
                .about("Print an example config which can be piped into website-stalker.yaml"),
        )
        .subcommand(App::new("init").about(
            "Initialize the current directory with a git repo and a config (website-stalker.yaml)",
        ))
        .subcommand(
            App::new("check")
                .about("Check if the config is fine but do not run")
                .arg(
                    Arg::new("print-yaml")
                        .long("print-yaml")
                        .help("Print out valid config as yaml"),
                )
                .arg(
                    Arg::new("rewrite-yaml")
                        .long("rewrite-yaml")
                        .help("Write valid config as website-stalker.yaml"),
                ),
        )
        .subcommand(
            App::new("run")
                .about("Stalk all the websites you specified")
                .arg(Arg::new("all").long("all").help("run for all sites"))
                .arg(
                    Arg::new("commit")
                        .long("commit")
                        .help("git commit changed files"),
                )
                .arg(
                    Arg::new("site filter")
                        .conflicts_with("all")
                        .required_unless_present("all")
                        .validator(Regex::new)
                        .value_name("SITE_FILTER")
                        .value_hint(clap::ValueHint::Other)
                        .help("Filter the sites to be run (case insensitive regular expression)"),
                ),
        )
}

#[test]
fn verify_app() {
    build().debug_assert();
}
