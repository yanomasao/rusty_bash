//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct IfCommand {
    pub text: String,
    pub if_scripts: Vec<Script>,
    pub then_scripts: Vec<Script>,
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
        self.if_scripts[0].exec(core);

        if core.vars["?"] != "0" {
            return;
        }

        self.then_scripts[0].exec(core);
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl IfCommand {
    fn new() -> IfCommand {
        IfCommand {
            text: String::new(),
            if_scripts: vec![],
            then_scripts: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<IfCommand> {
        let mut ans = Self::new();
        let mut if_script = None;
        if command::eat_inner_script(feeder, core, "if", vec!["then"], &mut if_script) {
            ans.text.push_str("if");
            ans.text.push_str(&if_script.as_mut().unwrap().get_text());
            ans.if_scripts.push(if_script.unwrap());
        }else{
            return None;
        }

        let mut then_script = None;
        if command::eat_inner_script(feeder, core, "then", vec!["fi"],  &mut then_script) {
            ans.text.push_str("then");
            ans.text.push_str(&then_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(2)); //fi
            ans.then_scripts.push(then_script.unwrap());

            loop {
                command::eat_blank_with_comment(feeder, core, &mut ans.text);
                if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                    break;
                }
            }
            Some(ans)
        }else{
            None
        }
    }
}
