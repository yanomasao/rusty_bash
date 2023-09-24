//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{IntoRawFd, RawFd};
use std::io::Error;
use crate::elements::io;
use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: String,
    pub left: String,
    left_fd: RawFd,
    left_backup: RawFd,
}

impl Redirect {
    pub fn connect(&mut self, restore: bool) -> bool {
        let result = match self.symbol.as_str() {
            "<" => self.redirect_simple_input_file(restore),
            ">" => self.redirect_simple_output_file(restore),
            ">&" => self.redirect_simple_output_fd(restore),
            ">>" => self.redirect_append_file(restore),
            _ => panic!("SUSH INTERNAL ERROR (Unknown redirect symbol)"),
        };

        if ! result {
            eprintln!("bash: {}: {}", &self.right, Error::last_os_error().kind());
        }

        result
    }

    fn set_left_fd(&mut self, default_fd: RawFd, restore: bool) {
        self.left_fd = if self.left.len() == 0 {
            default_fd
        }else{
            self.left.parse().unwrap()
        };

        if restore {
            self.left_backup = io::backup(self.left_fd);
        }
    }

    fn redirect_simple_input_file(&mut self, restore: bool) -> bool {
        self.set_left_fd(0, restore);
        if let Ok(fd) = File::open(&self.right) {
            io::replace(fd.into_raw_fd(), self.left_fd);
            true
        }else{
            false
        }
    }

    fn redirect_simple_output_file(&mut self, restore: bool) -> bool {
        self.set_left_fd(1, restore);
        if let Ok(fd) = File::create(&self.right) {
            io::replace(fd.into_raw_fd(), self.left_fd);
            true
        }else{
            false
        }
    }

    fn redirect_simple_output_fd(&mut self, restore: bool) -> bool {
        self.set_left_fd(1, restore);
        let right_fd = self.right.parse().unwrap();
        io::share(right_fd, self.left_fd)
    }

    fn redirect_append_file(&mut self, restore: bool) -> bool {
        self.set_left_fd(1, restore);
        if let Ok(fd) = OpenOptions::new().create(true).write(true)
                        .append(true).open(&self.right) {
            io::replace(fd.into_raw_fd(), self.left_fd);
            true
        }else{
            false
        }
    }

    pub fn restore(&mut self) {
        if self.left_backup >= 0 && self.left_fd >= 0 {
            io::replace(self.left_backup, self.left_fd);
        }
    }

    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
            symbol: String::new(),
            right: String::new(),
            left: String::new(),
            left_fd: -1,
            left_backup: -1,
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_redirect_symbol(core);
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();
        len != 0
    }

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        let len = feeder.scanner_word(core);
        ans.right = feeder.consume(len);
        ans.text += &ans.right.clone();
        len != 0
    }

    fn eat_left(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_nonnegative_integer(core);
        ans.left = feeder.consume(len);
        ans.text += &ans.left.clone();
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();
        let backup = feeder.clone();

        Self::eat_left(feeder, &mut ans, core);

        if Self::eat_symbol(feeder, &mut ans, core) &&
           Self::eat_right(feeder, &mut ans, core) {
            Some(ans)
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
