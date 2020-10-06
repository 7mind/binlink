use std::{
    path::Path,
};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs as ufs;

use toml_edit::{ArrayOfTables, Document, InlineTable, value, Value};


use std::path::PathBuf;
use std::env;
use env_logger::Builder;
use log;

use crate::cfg::GlobalConfig;

mod cfg;
mod cfgreader;
mod cli;
mod dl;
mod execv;


fn main() {
    Builder::new()
        .parse_filters(&env::var("BINLINK_LOG").unwrap_or_default())
        .init();

    let bin_path = std::env::current_exe()
        .map(|exe|
            exe.file_name()
                .map(OsString::from)
                .and_then(|s| s.to_str().map(|s| s.to_owned()))
        );
    let full_bin_path = std::env::current_exe();

    log::debug!("binlink: self={:#?}, path={:#?}", bin_path, full_bin_path);

    match (bin_path, full_bin_path) {
        (Ok(Some(e)), Ok(p)) => {
            match e.as_str() {
                "binlink" => {
                    let opts = cli::Args::make();
                    match opts.matches().subcommand_name() {
                        Some("link") => create_links(&p),
                        Some("example") => show_example(),
                        _ => opts.print_help(),

                    }
                }
                o => {
                    let config = cfgreader::get_config();
                    execv::do_passthrough(config, o);
                }
            }
        }
        o => {
            panic!("Cannot determine binary name, got: ${:#?}", o);
        }
    }

    // let result = download("https://github.com/graalvm/graalvm-ce-builds/releases/download/vm-20.2.0/graalvm-ce-java11-darwin-amd64-20.2.0.tar.gz");
    // match result {
    //     Ok(v) => println!("ok: {:?}", v),
    //     Err(e) => println!("error: {:?}", e),
    // }
}

fn create_links(p: &PathBuf) {
    let dir = "/usr/local/bin/binlinks";
    match fs::create_dir_all(dir) {
        Ok(_) => {
            println!("created: {}", dir);
        }
        Err(e) => {
            println!("cannot create {}: {:#?}", dir, e);
        }
    }
    let config = cfgreader::get_config();
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

fn show_example() {
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


fn make_inline_tbls(paths_arr: &mut ArrayOfTables) {
    for idx in 0..paths_arr.len() {
        let sub = paths_arr.get_mut(idx).expect("expected array element");
        let tgt_raw = sub.entry("target");

        tgt_raw.as_inline_table_mut().map(|e| e.fmt());
        let tgt_table = tgt_raw.as_table().expect("expected table");
        let mut tgt_inline = InlineTable::default();
        tgt_table.iter().for_each(|i| {
            let key = i.0;
            let value = i.1.as_value().expect("expected toml value").to_owned();
            tgt_inline.get_or_insert(key, value);
        });
        tgt_inline.fmt();


        let tgt_val = Value::InlineTable(tgt_inline);
        let tgt_item = value(tgt_val);
        *tgt_raw = tgt_item;
    }
}
