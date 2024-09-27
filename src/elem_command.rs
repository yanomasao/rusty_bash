//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use std::process;

pub struct Command {
    pub text: String,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) {
        if self.text == "exit\n" {
            process::exit(0);
        }

        let mut words = vec![];
        for w in self.text.trim_end().split(' ') {
            words.push(w);
        }

        println!("{:?}", words);
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        Some(Command { text: line })
    }
}
