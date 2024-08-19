//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder, Script};
use crate::elements::command;
use super::{Command, Redirect};

#[derive(Debug, Clone)]
pub struct IfCommand {
    pub text: String,
    pub if_elif_scripts: Vec<Script>,
    pub then_scripts: Vec<Script>,
    pub else_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for IfCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        for i in 0..self.if_elif_scripts.len() {
            self.if_elif_scripts[i].exec(core);
            if core.data.get_param("?") == "0" {
                self.then_scripts[i].exec(core);
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
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
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
            _ => error_message::internal(" (if parse error)"),
        }
    }

    fn set_script(word: &str, ans: &mut IfCommand, script: Option<Script>) {
        match word {
            "if" | "elif" => ans.if_elif_scripts.push(script.unwrap()),
            "then"        => ans.then_scripts.push(script.unwrap()),
            "else"        => ans.else_script = script,
            _ => error_message::internal(" (if parse error)"),
        };
    }

    fn eat_word_and_script(word: &str, feeder: &mut Feeder,
                           ans: &mut IfCommand, core: &mut ShellCore) -> bool {
        let mut s = None;
        let ends = Self::end_words(word);
        if ! command::eat_inner_script(feeder, core, word, ends, &mut s, false) {
            return false;
        }

        ans.text.push_str(word);
        ans.text.push_str(&s.as_ref().unwrap().get_text());
        Self::set_script(word, ans, s);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<IfCommand> {
        let mut ans = Self::new();
 
        let mut if_or_elif = "if";
        while Self::eat_word_and_script(if_or_elif, feeder, &mut ans, core) 
           && Self::eat_word_and_script("then", feeder, &mut ans, core) {

            Self::eat_word_and_script("else", feeder, &mut ans, core); //optional

            if feeder.starts_with("fi") { // If "else" exists, always it comes here.
                ans.text.push_str(&feeder.consume(2));
                break;
            }

            if_or_elif = "elif";
        }

        if ans.then_scripts.len() == 0 {
            return None;
        }

        command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
//        dbg!("{:?}", &ans);
        Some(ans)
    }
}
