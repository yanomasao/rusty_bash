//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;

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

    pub fn wait_process(&mut self, child: Pid) {}
}
