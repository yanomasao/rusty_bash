//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::elements::subword::Subword;
use crate::ShellCore;

fn is_lower(ch: char) -> bool { 'a' <= ch && ch <= 'z' }
fn is_upper(ch: char) -> bool { 'A' <= ch && ch <= 'Z' }
fn is_alphabet(ch: char) -> bool { is_lower(ch) || is_upper(ch) }
fn is_number(ch: char) -> bool { '0' <= ch && ch <= '9' }

pub fn eval(word: &mut Word, core: &ShellCore) {
    let mut skip = 0;
    for i in word.find("$") {
        if i < skip {
            continue;
        }

        let (len, s) = find_tail(&word.subwords[i+1..], core);
        if len > 0 {
            replace(&mut word.subwords[i..i+len+1], &s);
            skip = i + len + 1;
        }
    }
}

fn replace(subwords: &mut [Box<dyn Subword>], val: &String) {
    for sw in subwords.iter_mut() {
        sw.set_text("");
    }
    subwords[0].set_text(val);
}

fn find_tail(subwords: &[Box<dyn Subword>], core: &ShellCore) -> (usize, String) {
    if subwords.len() == 0 {
        return (0, "".to_string());
    }

    if subwords[0].get_text() == "{" {
        find_tail_brace(subwords, core)
    }else{
        find_tail_no_brace(subwords, core)
    }
}

fn find_tail_brace(subwords: &[Box<dyn Subword>], core: &ShellCore) -> (usize, String) {
    let mut name = String::new();
    for (i, sw) in subwords.iter().enumerate() {
        if i == 0 {
            continue;
        }

        if sw.get_text() == "}" {
            return (i+1, core.get_var(&name));
        }

        name += &sw.get_text();
    }

    (0, String::new())
}

fn find_tail_no_brace(subwords: &[Box<dyn Subword>], core: &ShellCore) -> (usize, String) {
    let mut ans = 0;
    let mut name = String::new();
    for sw in subwords {
        let text = sw.get_text().to_string();
        let mut len_as_name = param_name(&text);
        if len_as_name == 0 && name.len() == 0 {
            len_as_name = special_param(&text);
        }

        if len_as_name == 0 {
            break;
        }
        if len_as_name != text.len() {
            name += &text[0..len_as_name];
            return (ans+1, core.get_var(&name) + &text[len_as_name..]);
        }

        ans += 1;
        name += &text;
    }

    (ans, core.get_var(&name))
}

pub fn special_param(s: &str) -> usize {
    if let Some(_) = "$?".find(s) {
        1
    }else{
        0
    }
}

pub fn param_name(s: &str) -> usize {
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
