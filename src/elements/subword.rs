//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod single_quoted;
mod unquoted;

use crate::{ShellCore, Feeder};
use crate::elements::subword::single_quoted::SingleQuotedSubword;
use crate::elements::subword::unquoted::UnquotedSubword;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

impl Clone for Box::<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn merge(&mut self, right: &Box<dyn Subword>);
    fn unquote(&mut self);
    fn replace_parameter(&mut self, len: usize, val: &str);
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SingleQuotedSubword::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = UnquotedSubword::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
