//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct IfCommand {
    pub text: String,
    pub if_script: Option<Script>,
    pub then_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for IfCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork || pipe.is_connected() {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn run_command(&mut self, core: &mut ShellCore, _: bool) {
        self.if_script.as_mut()
            .expect("SUSH INTERNAL ERROR (no script)")
            .exec(core);

        if core.vars["?"] != "0" {
            return;
        }

        self.then_script.as_mut()
            .expect("SUSH INTERNAL ERROR (no script)")
            .exec(core);
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl IfCommand {
    fn new() -> IfCommand {
        IfCommand {
            text: String::new(),
            if_script: None,
            then_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<IfCommand> {
        let mut ans = Self::new();
            dbg!("{:?}", &ans);
        if command::eat_inner_script(feeder, core, "if", vec!["then"], &mut ans.if_script)
        && command::eat_inner_script(feeder, core, "then", vec!["fi"],  &mut ans.then_script) {
            ans.text.push_str("if");
            ans.text.push_str(&ans.if_script.as_mut().unwrap().get_text());
            ans.text.push_str("then");
            ans.text.push_str(&ans.then_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(2)); //fi

            loop {
                command::eat_blank_with_comment(feeder, core, &mut ans.text);
                if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                    break;
                }
            }
            dbg!("{:?}", &ans);
            Some(ans)
        }else{
            None
        }
    }
}
