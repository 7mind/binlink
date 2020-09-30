use clap::{App, ArgMatches};
use crate::cfgreader::{BINLINK_LOCAL_CONFIG_NAME_EVAR, BINLINK_BASE_CONFIG_PATH_EVAR};

pub struct Args {
    help: String
}

impl Args {

    pub fn make() -> Args {
        let formatted: String = format!(
            "export {} to override local config name\n\
             export {} to override global config path\n\
            ", BINLINK_LOCAL_CONFIG_NAME_EVAR, BINLINK_BASE_CONFIG_PATH_EVAR);

        Args { help: formatted }
    }

    pub fn matches(&self) -> ArgMatches {
        self.defn().get_matches()
    }

    pub fn print_help(&self) {
        self.defn().print_help().expect("Cannot print help");
    }

    fn defn(self: &Args) -> App {
        App::new("MyApp")
            .after_help(self.help.as_str())
            .subcommand(
                App::new("link")
                    .about("Create symlinks for all known binaries")
            )
            .subcommand(
                App::new("example")
                    .about("Show config example")
            ).clone()
    }

}

