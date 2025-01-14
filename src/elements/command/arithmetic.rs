//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::expr::arithmetic::ArithmeticExpr;

#[derive(Debug, Clone)]
pub struct ArithmeticCommand {
    pub text: String,
    expressions: Vec<ArithmeticExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for ArithmeticCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        let exit_status = match self.eval(core).as_deref() {
            Some("0") => "1",
            Some(_) => "0",
            None => "1",
        };
        core.data.set_param("?", exit_status );
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl ArithmeticCommand {
    fn new() -> ArithmeticCommand {
        ArithmeticCommand {
            text: String::new(),
            expressions: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        let mut ans = String::new();
        for a in &mut self.expressions {
            match a.eval(core) {
                Some(s) => ans = s,
                None    => return None,
            }
        }
        Some(ans)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("((") {
            return None;
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        if let Some(c) = ArithmeticExpr::parse(feeder, core) {
            if feeder.starts_with("))") {
                ans.text += &c.text;
                ans.text += &feeder.consume(2);
                ans.expressions.push(c);
                feeder.pop_backup();
                return Some(ans);
            }
        }
        feeder.rewind();
        return None;
    }
}
