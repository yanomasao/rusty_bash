//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

impl ShellCore {
    pub fn get_var(&self, key: &str) -> String {
        match self.vars.get(key) {
            Some(val) => val,
            None      => "",
        }.to_string()
    }

    pub fn get_var_ref(&self, key: &str) -> &str {
        match self.vars.get(key) {
            Some(val) => val,
            None      => "",
        }
    }

    pub fn set_var(&mut self, key: &str, val: &str) {
        self.vars.insert(key.to_string(), val.to_string());
    }
}
