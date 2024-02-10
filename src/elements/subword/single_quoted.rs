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
        if ! self.text.starts_with("\\") {
            return;
        }

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
        for i in 0..3 {
            let len = match i {
                0 => feeder.scanner_escaped_char(core),
                1 => feeder.scanner_subword_symbol(),
                2 => feeder.scanner_unquoted_subword(),
                _ => 0,
            };
            if len != 0 {
                return Some(Self::new( &feeder.consume(len) ));
            }
        }
        None
    }
}
