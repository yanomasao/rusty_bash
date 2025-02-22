//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder, Script};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone)]
pub struct ForCommand {
    text: String,
    name: String,
    has_in: bool,
    has_arithmetic: bool,
    values: Vec<Word>,
    arithmetics: Vec<Option<ArithmeticExpr>>,
    do_script: Option<Script>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for ForCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        core.loop_level += 1;

        let ok = match self.has_arithmetic {
            true  => self.run_with_arithmetic(core),
            false => self.run_with_values(core),
        };

        if ! ok && core.data.get_param("?") == "0" {
            core.data.set_param("?", "1");
        }

        core.loop_level -= 1;
        if core.loop_level == 0 {
            core.break_counter = 0;
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl ForCommand {
    fn eval_values(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        let mut ans = vec![];
        for w in &mut self.values {
            match w.eval(core) {
                Some(mut ws) => ans.append(&mut ws),
                None     => return None,
            }
        }

        Some(ans)
    }

    fn run_with_values(&mut self, core: &mut ShellCore) -> bool {
        let values = match self.has_in {
            true  => match self.eval_values(core) {
                Some(vs) => vs,
                None     => return false,
            },
            false => core.data.get_position_params(),
        };

        for p in values {
            if core.sigint.load(Relaxed) {
                return false;
            }

            core.data.set_param(&self.name, &p);

            self.do_script.as_mut()
                .expect(&error_message::internal_str("no script)"))
                .exec(core);

            if core.break_counter > 0 {
                core.break_counter -= 1;
                break;
            }
        }
        true
    }

    fn eval_arithmetic(a: &mut Option<ArithmeticExpr>, core: &mut ShellCore) -> (bool, String) {
        if a.is_none() {
            return (true, "1".to_string());
        }

        match a.clone().unwrap().eval(core) {
            Some(n) => return (true, n),
            None    => return (false, "0".to_string()), 
        }
    }

    fn run_with_arithmetic(&mut self, core: &mut ShellCore) -> bool {
        let (ok, _) = Self::eval_arithmetic(&mut self.arithmetics[0], core);
        if ! ok {
            return false;
        }

        loop {
            if core.sigint.load(Relaxed) {
                return false;
            }

            let (ok, val) = Self::eval_arithmetic(&mut self.arithmetics[1], core);
            if val == "0" {
                return ok;
            }

            self.do_script.as_mut()
                .expect(&error_message::internal_str("no script"))
                .exec(core);

            if core.break_counter > 0 {
                core.break_counter -= 1;
                break;
            }

            let (ok, _) = Self::eval_arithmetic(&mut self.arithmetics[2], core);
            if ! ok {
                return false;
            }
        }
        true
    }

    fn new() -> ForCommand {
        ForCommand {
            text: String::new(),
            name: String::new(),
            has_in: false,
            has_arithmetic: false,
            values: vec![],
            arithmetics: vec![],
            do_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        ans.name = feeder.consume(len);
        ans.text += &ans.name.clone();
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        true
    }

    fn eat_arithmetic(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("((") {
            return false;
        }
        ans.text += &feeder.consume(2);
        ans.has_arithmetic = true;
 
        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if feeder.len() == 0 {
                match feeder.feed_additional_line(core) {
                    true  => continue,
                    false => return false,
                }
            }

            let a = ArithmeticExpr::parse(feeder, core);
            if a.is_some() {
                ans.text += &a.as_ref().unwrap().text.clone();
            }
            ans.arithmetics.push(a);

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if feeder.starts_with(";") {
                if ans.arithmetics.len() >= 3 {
                    return false;
                }
                ans.text += &feeder.consume(1);
            }else if feeder.starts_with("))") {
                if ans.arithmetics.len() != 3 {
                    return false;
                }
                ans.text += &feeder.consume(2);
                return ans.arithmetics.len() == 3;
            }else {
                return false;
            }
        }
    }

    fn eat_in_part(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if ! feeder.starts_with("in") {
            return;
        }

        ans.text += &feeder.consume(2);
        ans.has_in = true;

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            match Word::parse(feeder, core, false) {
                Some(w) => {
                    ans.text += &w.text.clone();
                    ans.values.push(w);
                },
                None    => return,
            }
        }
    }

    fn eat_end(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        if feeder.starts_with(";") || feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("for") {
            return None;
        }
        let mut ans = Self::new();
        ans.text = feeder.consume(3);

        if Self::eat_name(feeder, &mut ans, core) {
            Self::eat_in_part(feeder, &mut ans, core);
        }else if ! Self::eat_arithmetic(feeder, &mut ans, core) {
            return None;
        }

        if ! Self::eat_end(feeder, &mut ans, core) {
            return None;
        }

        if feeder.len() == 0 && ! feeder.feed_additional_line(core) {
            return None;
        }

        if command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script, false) {
            ans.text.push_str("do");
            ans.text.push_str(&ans.do_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(4)); //done

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            Some(ans)
        }else{
            None
        }
    }
}
