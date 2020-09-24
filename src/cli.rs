use clap::{App, ArgMatches};

pub struct Args {}

impl Args {
    pub fn defn() -> ArgMatches {
        App::new("MyApp")
            .subcommand(
                App::new("link")
                    .about("Create symlinks for all known binaries")
            )
            .subcommand(
                App::new("example")
                    .about("Show config example")
            )
            .get_matches()
    }
}

