//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::{
    sys::wait::{self, WaitStatus},
    unistd::Pid,
};

pub struct ShellCore {
    pub history: Vec<String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let conf = ShellCore {
            history: Vec::new(),
        };

        conf
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_pid, status)) => status,
            Ok(unsupported) => {
                eprintln!("Unsupported: {:?}", unsupported);
                1
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                1
            }
        };
        eprintln!("終了ステータス： {}", exit_status);
    }
}
