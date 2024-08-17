//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;

use crate::{Feeder, ShellCore};
use crate::elements::subword;
use super::subword::Subword;

#[derive(Debug, Clone)] //Cloneも指定しておく
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn eval(&mut self) -> Option<Vec<String>> {
        let mut ws = brace_expansion::eval(self);
        Some( Self::make_args(&mut ws) )
    }

    fn make_args(words: &mut Vec<Word>) -> Vec<String> {
        words.iter()
              .map(|w| w.text.clone())
              .filter(|w| w.len() > 0)
              .collect()
    }

    pub fn new(subwords: Vec<Box::<dyn Subword>>) -> Word {
        Word {
            text: subwords.iter().map(|s| s.get_text()).collect(),
            subwords: subwords,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let mut subwords = vec![];
        while let Some(sw) = subword::parse(feeder, core) {
            subwords.push(sw);
        }

        let ans = Word::new(subwords);
        match ans.text.len() {
            0 => None,
            _ => Some(ans),
        }
    }
}
