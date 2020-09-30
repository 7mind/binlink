use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::cfg::{Config, GlobalConfig, LocalConfig};

pub const LOCAL_CONFIG_DEFAULT_NAME: &str = ".binlink.toml";
pub const BASE_CONFIG_DEFAULT_NAME: &str = "binlink.toml";
pub const BINLINK_LOCAL_CONFIG_NAME_EVAR: &str = "BINLINK_LOCAL_CONFIG_NAME";
pub const BINLINK_BASE_CONFIG_PATH_EVAR: &str = "BINLINK_BASE_CONFIG_PATH";

pub fn get_config() -> Config {
    let parsed_local = parse_local_config();
    let parsed_global = parse_base_config();

    Config {
        local: parsed_local,
        global: parsed_global,
    }
}

fn parse_base_config() -> Option<GlobalConfig> {
    let baseconfig = read_base_config();

    let parsed_global: Option<GlobalConfig> = match baseconfig {
        Some(c) => {
            Some(parse(c.as_path()))
        }
        None => { None }
    };
    parsed_global
}

fn parse_local_config() -> Option<LocalConfig> {
    let localconfig = read_local_config();

    let parsed_local: Option<LocalConfig> = match localconfig {
        Some(c) => {
            Some(parse(c.as_path()))
        }
        None => { None }
    };
    parsed_local
}

fn read_base_config() -> Option<PathBuf> {
    let baseconfig = match env::var(BINLINK_BASE_CONFIG_PATH_EVAR) {
        Ok(val) => {
            maybe_config_path(Some(PathBuf::from(val.as_str())))
        },
        Err(_) => {
            let home = dirs::home_dir().map(|h| h.join(".config").join("binlink"));
            maybe_config_in(home, BASE_CONFIG_DEFAULT_NAME)
        },
    };
    baseconfig
}

fn read_local_config() -> Option<PathBuf> {
    let local_config_name = match env::var(BINLINK_LOCAL_CONFIG_NAME_EVAR) {
        Ok(val) => val,
        Err(_) => String::from(LOCAL_CONFIG_DEFAULT_NAME),
    };

    let localconfig = maybe_config_in(std::env::current_dir().ok(), local_config_name.as_str());
    localconfig
}

fn maybe_config_in(base: Option<PathBuf>, name: &str) -> Option<PathBuf> {
    maybe_config_path(base.map(|p| p.as_path().join(name)))
}

fn maybe_config_path(path: Option<PathBuf>) -> Option<PathBuf> {
    path.and_then(|p| {
        if p.exists() {
            Some(p.as_path().to_owned())
        } else {
            None
        }
    })
}

pub fn parse<T>(path: &Path) -> T
    where
        T: DeserializeOwned,
{
    let mut config_toml = String::new();

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Could not find config file!");
        }
    };

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    match toml::from_str(config_toml.as_str()) {
        Ok(t) => t,
        Err(e) => panic!(format!("Error while deserializing config: {:#?}", e))
    }
}
