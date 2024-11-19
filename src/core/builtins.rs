use std::{env, fs, path::Path, process};

use super::ShellCore;

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }

    let exit_status = match core.vars["?"].parse::<i32>() {
        Ok(n) => n % 256,
        Err(_) => {
            eprintln!("sush: exit: {}: numeric argument required", core.vars["?"]);
            2
        }
    };
    process::exit(exit_status)
}

pub fn cd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 0 {
        eprintln!("Bug of this shell");
        return 1;
    }
    if args.len() > 2 {
        eprintln!("{}", "bash: cd: too many arguments");
        return 1;
    }

    if args.len() == 1 {
        let var = env::var("HOME").expect("HOME is not defined");
        args.push(var);
    } else if args.len() == 2 && args[1] == "-" {
        if let Some(old) = core.vars.get("OLDPWD") {
            args[1] = old.to_string();
        }
    }

    if let Ok(old) = env::current_dir() {
        core.vars
            .insert("OLDPWD".to_string(), old.display().to_string());
    }

    let path = Path::new(&args[1]);
    if env::set_current_dir(&path).is_ok() {
        if let Ok(full) = fs::canonicalize(path) {
            core.vars
                .insert("PWD".to_string(), full.display().to_string());
        }
        0
    } else {
        eprintln!("Not exist directory");
        1
    }
}
