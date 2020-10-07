use std::{
    io::Read,
};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;

use libc;
use crate::cfg::Config;


pub fn do_passthrough(config: Config, name: &str) -> () {
    let resolved = config.resolve();


    let bin: String = match resolved.bins.get(name) {
        Some(path) => {
            log::debug!("{} is configured: {}", name, path);
            find_main(name, path)
        }
        None => {
            log::debug!("{} not configured, matching on $PATH", name);
            find_fallback(name)
        }
    };

    log::info!("{} resolved as {}", name, bin);

    let mut f = BufReader::new(File::open(bin.clone()).expect("open failed"));
    let mut buffer = [0; 2];
    f.read_exact(&mut buffer).expect("cannot check file type");
    let has_shebang = buffer[0] == '#' as u8 && buffer[1] == '!' as u8;

    let env_args: Vec<String> = env::args().collect();
    let (_, just_args) = env_args.split_at(1);
    let just_args_vec = just_args.to_vec();

    let (sb_bin, args): (String, Vec<String>) = if has_shebang {
        let cmd: Vec<String> = vec![vec![bin], just_args_vec].into_iter().flatten().map(|s| format!("'{}'", s)).collect();
        (String::from("/bin/sh"), vec![vec![String::from("/bin/sh"), String::from("-c"), cmd.join(" ")]].into_iter().flatten().collect())
    } else {
        (bin.clone(), vec![vec![bin], just_args_vec].into_iter().flatten().collect())
    };

    log::info!("about to run: {}, argv: {:#?}", sb_bin, args);

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
