use std::{
    io::{Read},
    path::Path,
};
use std::env;
use std::ffi::{CString, OsString};
use std::fs::File;
use std::os::unix::fs as ufs;
use std::fs;
use std::path::PathBuf;

use libc;
use serde::de::DeserializeOwned;
use cfg::Config;

use crate::cfg::{GlobalConfig, LocalConfig};
use std::io::{BufReader, BufRead};

mod cfg;
mod cli;
mod dl;

use toml_edit::{Document, value, Value, Table, ArrayOfTables, Item, InlineTable};
use toml::value::Array;

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
    let cwd = env::current_dir().expect("cannot get cwd");

    let path = env::var_os("PATH").map(|p| {
        let split = env::split_paths(&p);
        env::join_paths(split.filter(|p| !p.to_str().expect("???").contains("binlinks"))).expect("???")
    });

    let result = which::which_in(name, path, &cwd);
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
    let resolved = config.resolve();


    let bin: String = match resolved.bins.get(name) {
        Some(path) => {
            find_main(name, path)
        }
        None => {
            find_fallback(name)
        }
    };

    let mut f = BufReader::new(File::open(bin.clone()).expect("open failed"));
    let mut buffer = [0; 2];
    f.read_exact(&mut buffer).expect("cannot check file type");
    let has_shebang = buffer[0] == '#' as u8 && buffer[1] == '!' as u8;

    let env_args: Vec<String> = env::args().collect();
    let (_, just_args) = env_args.split_at(1);
    let just_args_vec = just_args.to_vec();

    let (sb_bin, args): (String, Vec<String>) = if has_shebang {
        (String::from("/bin/sh"), vec![vec![String::from("/bin/sh"), bin], just_args_vec].into_iter().flatten().collect())
    } else {
        (bin.clone(), vec![vec![bin], just_args_vec].into_iter().flatten().collect())
    };

    let c_str_1 = CString::new(sb_bin.clone()).unwrap();


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
            panic!(format!("execve failed with code {}; command: `{} {}`", c, sb_bin, args.join(" ")))
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

fn get_config() -> Config {
    let localconfig = maybe_config(std::env::current_dir().ok(), ".binlink.toml");

    let home = dirs::home_dir().map(|h| h.join(".config").join("binlink"));
    let baseconfig = maybe_config(home, "binlink.toml");

    let parsed_local: Option<LocalConfig> = match localconfig {
        Some(c) => {
            Some(parse(c.as_path()))
        }
        None => { None }
    };

    let parsed_global: Option<GlobalConfig> = match baseconfig {
        Some(c) => {
            Some(parse(c.as_path()))
        }
        None => { Some(GlobalConfig::example()) }
    };

    let config = Config {
        local: parsed_local,
        global: parsed_global,
    };
    config
}

fn main() {
    let bin_path = std::env::current_exe()
        .map(|exe|
            exe.file_name()
                .map(OsString::from)
                .and_then(|s| s.to_str().map(|s| s.to_owned()))
        );
    let full_bin_path = std::env::current_exe();

    match (bin_path, full_bin_path) {
        (Ok(Some(e)), Ok(p)) => {
            match e.as_str() {
                "binlink" => {
                    let opts = cli::Args::defn();
                    match opts.subcommand_name() {
                        Some("link") => {
                            let dir = "/usr/local/bin/binlinks";
                            match fs::create_dir_all(dir) {
                                Ok(_) => {
                                    println!("created: {}", dir);
                                }
                                Err(e) => {
                                    println!("cannot create {}: {:#?}", dir, e);
                                }
                            }
                            let config = get_config();
                            let resolved = config.resolve();
                            resolved.names.iter().for_each(|k| {
                                let target = Path::new(dir).join(Path::new(k));
                                match fs::remove_file(&target) {
                                    _ => {}
                                }

                                let bin = Path::new(&p);
                                match ufs::symlink(&bin, &target) {
                                    Ok(_) => {
                                        println!("Created: {:#?} -> {:#?}", &target, &bin);
                                    }
                                    Err(e) => {
                                        panic!(format!("cannot symlink {}: {:#?}", k, e));
                                    }
                                }
                            })
                        }
                        Some("example") => {
                            let example = GlobalConfig::example();

                            match toml::to_string_pretty(&example) {
                                Ok(tout) => {
                                    let mut doc = tout.parse::<Document>().expect("invalid doc");

                                    {
                                        let bins_arr = doc["bins"].as_array_of_tables_mut().expect("xxx");
                                        make_inline_tbls(bins_arr)
                                    }

                                    {
                                        let paths_arr = doc["paths"].as_array_of_tables_mut().expect("xxx");
                                        make_inline_tbls(paths_arr)
                                    }
                                    println!("{}", doc.to_string());
                                }
                                Err(e) => {
                                    panic!(format!("error: {:#?}", e));
                                }
                            }
                        }
                        _ => panic!("Control endpoint is not implemented yet"),
                    }
                }
                o => {
                    let config = get_config();
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

fn make_inline_tbls(paths_arr: &mut ArrayOfTables) {
    for x in (0..paths_arr.len()) {
        let sub = paths_arr.get_mut(x).expect("");
        let tgt_raw = sub.entry("target");

        tgt_raw.as_inline_table_mut().map(|e| e.fmt());
        let tgt_table = tgt_raw.as_table().expect("");
        let mut tgt_inline = InlineTable::default();
        tgt_table.iter().for_each(|i| {
            let key = i.0;
            let value = i.1.as_value().expect("").to_owned();
            tgt_inline.get_or_insert(key, value);
        });
        tgt_inline.fmt();


        let mut tgt_val = Value::InlineTable(tgt_inline);
        let tgt_item = Item::Value(tgt_val);
        *tgt_raw = tgt_item;

    }
}
