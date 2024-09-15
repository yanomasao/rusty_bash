//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::expr::conditional::{ConditionalExpr, Elem};

#[derive(Debug, Clone)]
pub struct TestCommand {
    text: String,
    cond: Option<ConditionalExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        match self.cond.clone().unwrap().eval(core) {
            Ok(Elem::Ans(true))  => core.data.set_param("?", "0"),
            Ok(Elem::Ans(false)) => core.data.set_param("?", "1"),
            Err(err_msg)  => {
                error_message::print(&err_msg, core, true);
                core.data.set_param("?", "2");
            },
            _  => {
                error_message::print("unknown error", core, true);
                core.data.set_param("?", "2");
            },
        } 
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    fn new() -> TestCommand {
        TestCommand {
            text: String::new(),
            cond: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        match ConditionalExpr::parse(feeder, core) {
            Some(e) => {
                ans.text += &e.text.clone();
                ans.cond = Some(e);
            },
            None => return None,
        }

        if feeder.starts_with("]]") {
            ans.text += &feeder.consume(2);
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            return Some(ans);
        }
    
        None
    }
}
