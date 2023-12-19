//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct IfCommand {
    pub text: String,
    pub if_elif_scripts: Vec<Script>,
    pub then_scripts: Vec<Script>,
    pub else_script: Option<Script>,
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
        for (if_script, then_script) in self.if_elif_scripts
                                            .iter_mut()
                                            .zip(self.then_scripts.iter_mut()) {
            if_script.exec(core);
            if core.vars["?"] == "0" {
                then_script.exec(core);
                return;
            }
        }

        match self.else_script.as_mut() {
            Some(s) => s.exec(core),
            _ => {},
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl IfCommand {
    fn new() -> IfCommand {
        IfCommand {
            text: String::new(),
            if_elif_scripts: vec![],
            then_scripts: vec![],
            else_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    fn end_words(word: &str) -> Vec<&str> {
        match word {
            "if" | "elif" => vec!["then"],
            "then" => vec!["fi", "else", "elif"],
            "else" => vec!["fi"],
            _ => panic!("SUSH INTERNAL ERROR (if parse error)"),
        }
    }

    fn eat_script(word: &str, feeder: &mut Feeder, ans: &mut IfCommand, core: &mut ShellCore) -> bool {
        let mut s = None;
        if command::eat_inner_script(feeder, core, word, Self::end_words(word), &mut s) {
            ans.text.push_str(word);
            ans.text.push_str(&s.as_mut().unwrap().get_text());

            match word {
                "if" | "elif" => ans.if_elif_scripts.push(s.unwrap()),
                "then"        => ans.then_scripts.push(s.unwrap()),
                "else"        => ans.else_script = s,
                _ => panic!("SUSH INTERNAL ERROR (if parse error)"),
            };

            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<IfCommand> {
        let mut ans = Self::new();

        if ! Self::eat_script("if", feeder, &mut ans, core) {
            return None;
        }

        loop {
            if ! Self::eat_script("then", feeder, &mut ans, core) {
                return None;
            }

            if feeder.starts_with("fi") {
                ans.text.push_str(&feeder.consume(2));
                break;
            }else if feeder.starts_with("else") {
                if ! Self::eat_script("else", feeder, &mut ans, core) {
                    return None;
                }
                ans.text.push_str(&feeder.consume(2)); //fi
                break;
            }else if feeder.starts_with("elif") {
                if ! Self::eat_script("elif", feeder, &mut ans, core) {
                    return None;
                }
            }else{
                panic!("SUSH INTERNAL ERROR (parse error on if command)");
            }
        }

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                break;
            }
        }
        Some(ans)
    }
}
