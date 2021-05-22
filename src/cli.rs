use clap::{App, AppSettings, Arg, SubCommand};
use regex::Regex;

pub fn build() -> App<'static, 'static> {
    App::new("Website Stalker")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("example-config")
                .about("Prints an example config which can be piped into website-stalker.yaml"),
        )
        .subcommand(
            SubCommand::with_name("check").about("check if the config is fine but do not run"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("stalk all the websites you specified")
                .arg(Arg::with_name("all").long("all").help("run for all sites"))
                .arg(
                    Arg::with_name("commit")
                        .long("commit")
                        .help("git commit changed files"),
                )
                .arg(
                    Arg::with_name("site filter")
                        .conflicts_with("all")
                        .required_unless("all")
                        .validator(|v| match Regex::new(&v) {
                            Ok(_) => Ok(()),
                            Err(err) => Err(format!("{}", err)),
                        })
                        .help("filter the rules to be run"),
                ),
        )
}
