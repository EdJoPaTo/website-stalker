use clap::{command, value_parser, Arg, Command, ValueHint};
use regex::Regex;

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build() -> Command<'static> {
    command!()
        .name("Website Stalker")
        .subcommand_required(true)
        .subcommand(
            Command::new("example-config")
                .about("Print an example config which can be piped into website-stalker.yaml"),
        )
        .subcommand(Command::new("init").about(
            "Initialize the current directory with a git repo and a config (website-stalker.yaml)",
        ))
        .subcommand(
            Command::new("check")
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
            Command::new("run")
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
                        .takes_value(true)
                        .value_parser(value_parser!(Regex))
                        .value_hint(ValueHint::Other)
                        .value_name("SITE_FILTER")
                        .help("Filter the sites to be run (case insensitive regular expression)"),
                ),
        )
}

#[test]
fn verify() {
    build().debug_assert();
}
