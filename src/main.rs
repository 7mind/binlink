use std::{
    io::{Read},
    path::Path,

};
use std::env;
use std::ffi::{CString, OsString};
use std::fs::File;
use std::path::PathBuf;

use libc;
use serde::de::DeserializeOwned;

use cfg::Config;

use crate::cfg::GlobalConfig;

mod cfg;
mod cli;
mod dl;


struct UnsafeStrVec {
    base: Vec<CString>,
    tmp: Vec<*const i8>,
}

fn make_cstring_array(argv: Vec<String>) -> UnsafeStrVec {
    let cstr_argv: Vec<_> = argv.iter()
        .map(|arg| CString::new(arg.as_str()).unwrap())
        .collect();

    let mut p_argv: Vec<_> = cstr_argv.iter() // do NOT into_iter()
        .map(|arg| arg.as_ptr())
        .collect();

    p_argv.push(std::ptr::null());

    let out = UnsafeStrVec {
        base: cstr_argv,
        tmp: p_argv,
    };


    return out;
}


fn make_env() -> Vec<String> {
    return env::vars().map(|(key, value)| format!("{}={}", key, value)).collect();
}

fn find_fallback(name: &str) -> String {
    let result = which::which(name);
    match result {
        Ok(bin) => {
            match bin.to_str() {
                Some(path) => { path.to_owned() }
                _ =>
                    panic!(format!("Cannot get binary name for: {}", name))
            }
        }
        _ => {
            panic!(format!("Cannot find binary in $PATH: {}", name))
        }
    }
}

fn find_main(name: &str, target: &str) -> String {
    let result = which::which_in(name, Some(target), target);
    match result {
        Ok(bin) => {
            match bin.to_str() {
                Some(path) => { path.to_owned() }
                _ =>
                    panic!(format!("Cannot get binary name for: {}", name))
            }
        }
        _ => {
            panic!(format!("Cannot find binary in $PATH: {}", name))
        }
    }
}

fn do_passthrough(config: Config, name: &str) -> () {
    //let config = config.into_inner();
    let resolved = config.resolve();

    let bin: String = match resolved.bins.get(name) {
        Some(path) => {
            println!("resolved!");
            find_main(name, path)
        }
        None => {
            find_fallback(name)
        }
    };

    println!("target: {}", bin);
    let c_str_1 = CString::new(bin.clone()).unwrap();

    let args: Vec<String> = env::args().collect();
    //let args = ; //vec![c_str_1.to_str().unwrap().to_string(), "-c".to_string(), "env".to_string()];

    let argv = make_cstring_array(args.clone());
    let envp = make_cstring_array(make_env());

    let out = unsafe {
        libc::execve(c_str_1.as_ptr(), argv.tmp.as_ptr(), envp.tmp.as_ptr())
    };

    match out {
        0 => {
            panic!("Impossible: Launcher continued after successful execve")
        }
        c => {
            panic!(format!("execve failed with code {}; command: `{} {}`", c, bin, args.join(" ")))
        }
    }
}

fn maybe_config(base: Option<PathBuf>, name: &str) -> Option<PathBuf> {
    base.map(|p| p.as_path().join(name)).and_then(|p| {
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
        Err(_)  => {
            panic!("Could not find config file, using default!");
            //return Config::new();
        }
    };

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    // let mut parser = Parser::new(&config_toml);
    // let toml = parser.parse();
    //
    // if toml.is_none() {
    //     for err in &parser.errors {
    //         let (loline, locol) = parser.to_linecol(err.lo);
    //         let (hiline, hicol) = parser.to_linecol(err.hi);
    //         println!("{}:{}:{}-{}:{} error: {}",
    //                  path.display(), loline, locol, hiline, hicol, err.desc);
    //     }
    //     panic!("Exiting server");
    // }
    //
    // let config = toml::Value::Table(toml.unwrap());

    match toml::from_str(config_toml.as_str()) {
        Ok(t) => t,
        Err(e) => panic!(format!("Error while deserializing config: {:#?}", e))
    }
}

fn main() {

    let home = dirs::home_dir().map(|h| h.join(".config").join("binlink"));
    // println!("home: {:#?}", home.clone());
    //



    let localconfig = maybe_config(std::env::current_dir().ok(), ".binlink.toml");
    let baseconfig = maybe_config(home, "binlink.toml");

    // let parsed_local: Option<LocalConfig> = match localconfig {
    //     Some(c) => {
    //         Some(parse(c.as_path()))
    //     }
    //     None => {None}
    // };
    //
    // let parsed_global: Option<GlobalConfig> = match baseconfig {
    //     Some(c) => {
    //         println!("base config: {}", c.display());
    //         Some(parse(c.as_path()))
    //     }
    //     None => {None}
    // };

    // let config = Config {
    //     local: parsed_local,
    //     global: parsed_global,
    // };

    let config = Config {
        local: None,
        global: Some(GlobalConfig::example()),
    };


    let bin_path = std::env::current_exe()
        .map(|exe|
            exe.file_name()
                .map(OsString::from)
                .and_then(|s| s.to_str().map(|s| s.to_owned()))
        );



    match bin_path.map(|e| e.map(|s| s)) {
        Ok(Some(e)) => {
            match e.as_str() {
                "binlink" => {
                    let opts = cli::Args::defn();
                    match opts.subcommand_name() {
                        Some("link") => {
                            let resolved = config.resolve();
                            resolved.bins.iter().for_each(|(k, _)| println!("{}", k))
                        },
                        Some("example") => {
                            let example = GlobalConfig::example();
                            match toml::to_string_pretty(&example) {
                                Ok(tout) => {
                                    println!("{}", tout);
                                }
                                Err(e) => {
                                    panic!(format!("error: {:#?}", e));
                                }
                            }
                        },
                        _ => panic!("Control endpoint is not implemented yet"),
                    }
                }
                o => {
                    do_passthrough(config, o);
                }
            }
        }
        _ => {
            panic!("Cannot determine binary name");
        }
    }

    // let result = download("https://github.com/graalvm/graalvm-ce-builds/releases/download/vm-20.2.0/graalvm-ce-java11-darwin-amd64-20.2.0.tar.gz");
    // match result {
    //     Ok(v) => println!("ok: {:?}", v),
    //     Err(e) => println!("error: {:?}", e),
    // }
}
