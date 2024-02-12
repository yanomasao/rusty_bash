//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::elements::subword::Subword;
use crate::ShellCore;

pub fn eval(word: &mut Word, core: &ShellCore) {
    for i in word.find("$") {
        let (len, s) = find_tail(&word.subwords[i+1..], core);
        if len > 0 {
            replace(&mut word.subwords[i..i+len+1], &s);
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
    if subwords.len() > 0 && subwords[0].get_text() == "{" {
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
            match core.vars.get(&name) {
                Some(v) => return (i+1, v.clone()), 
                None => return (i+1, "".to_string()),
            };
        }

        name += &sw.get_text();
    }

    (0, String::new())
}

fn find_tail_no_brace(subwords: &[Box<dyn Subword>], core: &ShellCore) -> (usize, String) {
    let mut ans = 0;
    let mut nm = String::new();
    for sw in subwords {
        let text = sw.get_text().to_string();
        let len_as_name = name(&text);

        if len_as_name == 0 {
            break;
        }
        if len_as_name != text.len() {
            nm += &text[0..len_as_name];
            match core.vars.get(&nm) {
                Some(v) => return (ans+1, v.clone() + &text[len_as_name..]), 
                None => return (ans+1, text[len_as_name..].to_string()),
            };
        }

        ans += 1;
        nm += &text;
    }

    match core.vars.get(&nm) {
        Some(v) => (ans, v.clone()), 
        None => (ans, "".to_string()),
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
