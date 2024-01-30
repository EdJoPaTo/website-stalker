use clap::{Parser, ValueHint};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(about, version)]
pub enum Cli {
    /// Print an example configuration file which can be piped into website-stalker.yaml
    ExampleConfig,

    /// Initialize the current directory with a git repository and a configuration file (website-stalker.yaml)
    Init,

    /// Check if the configuration is fine but do not run
    Check,

    /// Stalk all the websites you specified
    Run {
        /// Run for all sites
        #[arg(long)]
        all: bool,

        /// git commit changed files
        #[arg(long)]
        commit: bool,

        /// Filter the sites to be run (case insensitive regular expression)
        #[arg(
            value_hint = ValueHint::Other,
            conflicts_with = "all",
            required_unless_present = "all",
        )]
        site_filter: Option<Regex>,
    },
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
