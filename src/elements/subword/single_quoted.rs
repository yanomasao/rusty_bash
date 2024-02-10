//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct SingleQuotedSubword {
    pub text: String,
}

impl Subword for SingleQuotedSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text().clone();
    }

    fn unquote(&mut self) {
        dbg!("{:?}", &self);
        self.text.pop();
        self.text.remove(0);
    }
}

impl SingleQuotedSubword {
    fn new(s: &str) -> SingleQuotedSubword {
        SingleQuotedSubword {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SingleQuotedSubword> {
        if ! feeder.starts_with("'") {
            return None;
        }

        let len = feeder.scanner_single_quoted_subword(core);
        if len != 0 {
            return Some(Self::new( &feeder.consume(len) ));
        }

        None
    }
}
