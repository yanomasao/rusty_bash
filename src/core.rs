//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;

use nix::{
    sys::wait::{self, WaitStatus},
    unistd::Pid,
};

pub struct ShellCore {
    pub history: Vec<String>,
    pub vars: HashMap<String, String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            history: Vec::new(),
            vars: HashMap::new(),
        };

        core.vars.insert("?".to_string(), "0".to_string());
        core
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_pid, status)) => status,
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128 + signal as i32
            }
            Ok(unsupported) => {
                eprintln!("Unsupported: {:?}", unsupported);
                1
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        };
        // eprintln!("終了ステータス： {}", exit_status);
        self.vars.insert("?".to_string(), exit_status.to_string());
    }
}
