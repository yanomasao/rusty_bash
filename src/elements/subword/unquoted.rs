//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};

#[derive(Debug,Clone)]
pub struct UnquotedSubword {
    pub text: String,
}

impl UnquotedSubword {
    fn new(s: &str) -> UnquotedSubword {
        UnquotedSubword {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<UnquotedSubword> {
        let len = feeder.scanner_unquoted_subword(core);
        if len == 0 {
            return None;
        }

        Some(UnquotedSubword::new(&feeder.consume(len)))
    }
}
