//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    if word.subwords.len() == 0
    || word.subwords[0].get_text() != "~" {
        return;
    }

    let mut text = String::new();
    let mut pos = 1;
    for sw in &word.subwords[1..] {
        if sw.get_text() == "/" {
            break;
        }
        text += &sw.get_text();
        pos += 1;
    }

    eprintln!("{}, {}", text, pos);
}

