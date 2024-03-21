use clap::{Parser, ValueHint};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(about, version)]
pub enum Cli {
    /// Print an example configuration file which can be piped into website-stalker.yaml
    ExampleConfig,

    /// Initialize the current directory with a git repository and a configuration file (website-stalker.yaml)
    #[command(hide = true)]
    Init,

    /// Check if the configuration is fine but do not run
    #[command(hide = true)]
    Check,

    /// Stalk all the websites you specified
    Run {
        /// Run for all sites
        #[arg(long)]
        all: bool,

        /// git commit changed files
        #[arg(long)]
        commit: bool,

        /// Used as the From header in the web requests.
        ///
        /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From>
        ///
        /// The idea here is to provide a way for a website host to contact whoever is doing something to their web server.
        /// As this tool is self-hosted and can be run as often as the user likes this can annoy website hosts.
        /// While this tool is named "stalker" and is made to track websites it is not intended to annoy people.
        ///
        /// Can also be specified in the config instead.
        #[arg(
            long,
            env = "WEBSITE_STALKER_FROM",
            value_hint = ValueHint::EmailAddress,
        )]
        from: Option<String>,

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
