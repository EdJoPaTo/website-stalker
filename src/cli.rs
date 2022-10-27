use clap::{Parser, ValueHint};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Print an example config which can be piped into website-stalker.yaml
    ExampleConfig,

    /// Initialize the current directory with a git repo and a config (website-stalker.yaml)
    Init,

    /// Check if the config is fine but do not run
    Check {
        /// Print out valid config as yaml
        #[arg(long)]
        print_yaml: bool,

        /// Write valid config as website-stalker.yaml
        #[arg(long)]
        rewrite_yaml: bool,
    },

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
