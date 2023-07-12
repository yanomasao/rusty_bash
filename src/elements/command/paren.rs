//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::Command;
use crate::elements::command;
use super::{Pipe, Redirect};

#[derive(Debug)]
pub struct ParenCommand {
    pub text: String,
    pub script: Option<Script>,
    pub redirects: Vec<Redirect>,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) {
        match self.script {
            Some(ref mut s) => s.fork_exec(core, pipe),
            _               => panic!("SUSH INTERNAL ERROR (ParenCommand::exec)"),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl ParenCommand {
    fn new() -> ParenCommand {
        ParenCommand {
            text: String::new(),
            script: None,
            redirects: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        match Script::parse_nested(feeder, core, "(") {
            Some(s) => {
                let mut ans = Self::new();
                ans.text = "(".to_string() + &s.text.clone() + &feeder.consume(1);
                ans.script = Some(s);

                while command::eat_redirect(feeder, core, &mut ans.redirects) {}

                Some(ans)
            },
            None => None, 
        }
    }
}
