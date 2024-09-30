//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{self, execvp, ForkResult};

use crate::{Feeder, ShellCore};
use std::{ffi::CString, process};

pub struct Command {
    pub text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
}

impl Command {
    pub fn exec(&mut self, core: &mut ShellCore) {
        if self.text == "exit\n" {
            process::exit(0);
        }

        // println!("{:?}", execvp(&self.cargs[0], &self.cargs));
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Child) => {
                let err = unistd::execvp(&self.cargs[0], &self.cargs);
                println!("Failed to execute. {:?}", err);
                process::exit(127);
            }
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
                text: line,
                args,
                cargs,
            })
        } else {
            None
        }
    }
}
