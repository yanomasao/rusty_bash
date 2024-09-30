//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::{
    errno::Errno,
    unistd::{self, execvp, ForkResult},
};

use crate::{Feeder, ShellCore};
use std::{ffi::CString, process};

pub struct Command {
    _text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
}

impl Command {
    pub fn exec(&mut self, core: &mut ShellCore) {
        if self.args[0] == "exit" {
            eprintln!("exit");
            if self.args.len() > 1 {
                // process::exit(self.args[1].parse::<i32>().unwrap());
                core.vars.insert("?".to_string(), self.args[1].clone());
            }
            // process::exit(0);
            let exit_status = match core.vars["?"].parse::<i32>() {
                Ok(n) => {
                    if 0 <= n && n <= 255 {
                        n
                    } else {
                        n % 256
                    }
                }
                Err(_) => {
                    eprintln!("sush: exit: {}: numeric argument required", core.vars["?"]);
                    2
                }
            };
            process::exit(exit_status);
        }

        // println!("{:?}", execvp(&self.cargs[0], &self.cargs));
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Child) => match unistd::execvp(&self.cargs[0], &self.cargs) {
                Err(Errno::EACCES) => {
                    println!("sush: {}: Permission denied", &self.args[0]);
                    process::exit(126);
                }
                Err(Errno::ENOENT) => {
                    println!("{}: command not found", &self.args[0]);
                    process::exit(127);
                }
                Err(err) => {
                    println!("Failed to execute. {:?}", err);
                    process::exit(127);
                }
                _ => (),
            },
            Ok(ForkResult::Parent { child }) => {
                eprintln!("PID{}の親です", child);
                core.wait_process(child);
            }
            Err(err) => panic!("Failed to fork. {:?}", err),
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        let args: Vec<String> = line.trim_end().split(' ').map(|s| s.to_string()).collect();
        let cargs: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect();

        if !args.is_empty() {
            Some(Command {
                _text: line,
                args,
                cargs,
            })
        } else {
            None
        }
    }
}
