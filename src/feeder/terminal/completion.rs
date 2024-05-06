//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::builtins::completion;
use crate::feeder::terminal::Terminal;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

fn common_length(chars: &Vec<char>, s: &String) -> usize {
    let max_len = chars.len();
    for (i, c) in s.chars().enumerate() {
        if i >= max_len || chars[i] != c {
            return i;
        }
    }
    max_len
}

fn common_string(paths: &Vec<String>) -> String {
    if paths.len() == 0 {
        return "".to_string();
    }

    let ref_chars: Vec<char> = paths[0].chars().collect();
    let mut common_len = ref_chars.len();

    for path in &paths[1..] {
        let len = common_length(&ref_chars, &path);
        common_len = std::cmp::min(common_len, len);
    }

    ref_chars[..common_len].iter().collect()
}

impl Terminal {
    pub fn completion(&mut self, core: &mut ShellCore, double_tab: bool) {
        let input = self.get_string(self.prompt.chars().count());
        let words: Vec<String> = input.split(" ").map(|e| e.to_string()).collect();
        if words.len() == 0 || words.last().unwrap() == "" {
            self.cloop();
            return;
        }

        let last = words.last().unwrap().clone();
        let mut last_tilde_expanded = last.clone();

        if last.starts_with("~/") {
            self.tilde_prefix = "~/".to_string();
            self.tilde_path = core.data.get_param_ref("HOME").to_string() + "/";
            last_tilde_expanded = last.replacen(&self.tilde_prefix, &self.tilde_path, 1);
        }else{
            self.tilde_prefix = String::new();
            self.tilde_path = String::new();
        }

        let mut command_pos = 0;
        for w in &words {
            match w.find("=") {
                None => break,
                _    => command_pos +=1,
            }
        }
        let search_command = command_pos == words.len()-1;

        let mut args = vec!["".to_string(), "".to_string(), last_tilde_expanded.to_string()];
        let list = match search_command {
            true  => completion::compgen_c(core, &mut args),
            false => completion::compgen_f(core, &mut args),
        };

        let list_output: Vec<String> = list.iter().map(|p| p.replacen(&self.tilde_path, &self.tilde_prefix, 1)).collect();

        if double_tab {
            self.show_list(&list_output);
        }

        if list.len() == 1 {
            let tail = match Path::new(&list[0]).is_dir() {
                true  => "/",
                false => " ",
            };
            self.replace_input(&(list_output[0].to_string() + tail), &last);
            return;
        }

        let common = common_string(&list);
        self.replace_input(&common, &last);
    }

    /*
    pub fn command_completion(&mut self, target: &String, core: &mut ShellCore) {
        let mut args = vec!["".to_string(), "".to_string(), target.to_string()];
        let comlist = completion::compgen_c(core, &mut args);

        match comlist.len() {
            0 => self.cloop(),
            1 => {
                let last = match Path::new(&comlist[0]).is_dir() {
                    true  => "/",
                    false => " ",
                };
                self.replace_input(&(comlist[0].to_string() + last), &target)
            },
            _ => self.show_list(&comlist),
        }
    }

    pub fn file_completion(&mut self, target: &String, core: &mut ShellCore, double_tab: bool) {
        let mut target_tilde = target.to_string();
        if target.starts_with("~/") {
            self.tilde_prefix = "~/".to_string();
            self.tilde_path = core.data.get_param_ref("HOME").to_string() + "/";
            target_tilde = target_tilde.replacen(&self.tilde_prefix, &self.tilde_path, 1);
        }else{
            self.tilde_prefix = String::new();
            self.tilde_path = String::new();
        }

        let mut args = vec!["".to_string(), "".to_string(), target.to_string()];
        let paths = completion::compgen_f(core, &mut args);
        core.data.set_array("COMPREPLY", &paths);

        match paths.len() {
            0 => self.cloop(),
            1 => {
                let last = match Path::new(&paths[0]).is_dir() {
                    true  => "/",
                    false => " ",
                };
                self.replace_input(&(paths[0].to_string() + last), &target)
            },
            _ => self.file_completion_multicands(&target_tilde, &paths, double_tab),
        }
    }
    */

    fn show_list(&mut self, list: &Vec<String>) {
        eprintln!("\r");

        let widths: Vec<usize> = list.iter()
                                     .map(|p| UnicodeWidthStr::width(p.as_str()))
                                     .collect();
        let max_entry_width = widths.iter().max().unwrap_or(&1000) + 1;

        let col_num = Terminal::size().0 / max_entry_width;
        if col_num == 0 {
            list.iter().for_each(|p| print!("{}\r\n", &p));
            self.rewrite(true);
            return;
        }

        let row_num = (list.len()-1) / col_num + 1;

        for row in 0..row_num {
            for col in 0..col_num {
                let i = col*row_num + row;
                if i >= list.len() {
                    continue;
                }

                let space_num = max_entry_width - widths[i];
                let s = String::from_utf8(vec![b' '; space_num]).unwrap();
                print!("{}{}", list[i], &s);
            }
            print!("\r\n");
        }
        self.rewrite(true);
    }

    /*
    pub fn file_completion_multicands(&mut self, dir: &String,
                                      paths: &Vec<String>, double_tab: bool) {
        let common = common_string(&paths);
        if common.len() == dir.len() {
            match double_tab {
                true => self.show_path_candidates(&dir.to_string(), &paths),
                false => self.cloop(),
            }
            return;
        }
        self.replace_input(&common, &dir);
    }

    pub fn show_path_candidates(&mut self, dir: &String, paths: &Vec<String>) {
        let ps = if dir.chars().last() == Some('/') && dir.len() > 1 {
            paths.iter()
                 .map(|p| p.replacen(dir, "", 1)
                 .replacen(&self.tilde_path, &self.tilde_prefix, 1))
                 .collect()
        }else{
            paths.iter()
                 .map(|p| p.replacen(&self.tilde_path, &self.tilde_prefix, 1))
                 .collect()
        };

        self.show_list(&ps);
    }
    */

    fn replace_input(&mut self, path: &String, last: &str) {
        let last_char_num = last.chars().count();
        let len = self.chars.len();
        let mut path_chars = path.to_string();

        if last.starts_with("./") {
            path_chars.insert(0, '/');
            path_chars.insert(0, '.');
        }
        
        path_chars = path_chars.replacen(&self.tilde_path, &self.tilde_prefix, 1);

        self.chars.drain(len - last_char_num..);
        self.chars.extend(path_chars.chars());
        self.head = self.chars.len();
        self.rewrite(false);
    }
}
