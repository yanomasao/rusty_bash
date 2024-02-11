//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::ShellCore;

pub fn eval(word: &mut Word, core: &ShellCore) {
    concatenate_subwords(word); 

    let mut dollar = false;
    for sw in word.subwords.iter_mut() {
        if dollar {
            let text = sw.get_text().to_string();
            let len_as_name = name(&text);
            let name = text[..len_as_name].to_string();

            let val = match core.vars.get(&name) {
                Some(v) => v.clone(), 
                None => "".to_string(),
            };

            sw.replace_parameter(len_as_name, &val);

            dollar = false;
            continue;
        }

        if sw.get_text() == "$" {
            sw.replace_parameter(1, "");
            dollar = true;
        }
    }
}

fn is_lower(ch: char) -> bool { 'a' <= ch && ch <= 'z' }
fn is_upper(ch: char) -> bool { 'A' <= ch && ch <= 'Z' }
fn is_alphabet(ch: char) -> bool { is_lower(ch) || is_upper(ch) }
fn is_number(ch: char) -> bool { '0' <= ch && ch <= '9' }

pub fn name(s: &str) -> usize {
    if s.len() == 0 {
        return 0;
    }

    let head_ch = s.chars().nth(0).unwrap();
    if ! is_alphabet(head_ch) && head_ch != '_' {
        return 0;
    }

    let mut ans = 1;
    for ch in s[1..].chars() {
        if is_alphabet(ch) || is_number(ch) || ch == '_' {
            ans += 1;
        }else{
            break;
        }
    }

    ans
}

fn connectable(s: &str) -> bool {
    if s.len() == 0 {
        return true;
    }

    let c = s.chars().nth(0).unwrap();
    match "'$\\\"".find(c) {
        Some(_) => false,
        None    => true,
    }
}

fn concatenate_subwords(word: &mut Word) {
    if word.subwords.len() == 0 {
        return;
    }

    let mut ans = vec![];
    let mut left = word.subwords.remove(0);
    while word.subwords.len() != 0 {
        let right = word.subwords.remove(0);
        if connectable(left.get_text()) 
        && connectable(right.get_text()) {
            left.merge(&right);
        }else{
            ans.push(left);
            left = right;
        }
    }
    ans.push(left);

    word.subwords = ans;
}
