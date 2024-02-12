//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
mod parameter_expansion;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}


impl Word {
    pub fn eval(&mut self, core: &ShellCore) -> Vec<String> {
        let mut ws = brace_expansion::eval(self);
        ws.iter_mut().for_each(|w| w.expansion(core));
        ws.iter().map(|w| w.text.clone()).filter(|arg| arg.len() > 0).collect()
    }

    fn expansion(&mut self, core: &ShellCore) {
        parameter_expansion::eval(self, core);
        self.unquote();
        self.connect_text();
    }

    fn unquote(&mut self) {
        self.subwords.iter_mut().for_each(|w| w.unquote());
    }

    fn connect_text(&mut self) {
        self.text = self.subwords.iter()
                    .map(|s| s.get_text().clone())
                    .collect::<String>();
    }

    fn find(&mut self, s: &str) -> Vec<usize> {
        self.subwords.iter()
            .enumerate()
            .filter(|e| e.1.get_text() == s)
            .map(|e| e.0)
            .collect()
    }

    pub fn new() -> Word {
        Word {
            text: String::new(),
            subwords: vec![],
        }
    }

    fn push(&mut self, subword: &Box<dyn Subword>) {
        self.text += &subword.get_text().to_string();
        self.subwords.push(subword.clone());
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let mut ans = Word::new();
        while let Some(sw) = subword::parse(feeder, core) {
            ans.push(&sw);
        }

        if ans.text.len() == 0 {
            None
        }else{
            Some(ans)
        }
    }
}
