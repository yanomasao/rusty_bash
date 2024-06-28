//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod data;
pub mod history;
pub mod jobtable;

use self::data::Data;
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::{io, env, path, process};
use nix::{fcntl, unistd};
use nix::sys::{signal, wait};
use nix::sys::signal::{Signal, SigHandler};
use nix::sys::wait::WaitStatus;
use nix::sys::time::{TimeSpec, TimeVal};
use nix::time;
use nix::time::ClockId;
use nix::unistd::Pid;
use crate::core::jobtable::JobEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

pub struct ShellCore {
    pub data: Data,
    rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
    pub sigint: Arc<AtomicBool>,
    pub read_stdin: bool,
    pub is_subshell: bool,
    pub source_function_level: i32,
    pub loop_level: i32,
    pub break_counter: i32,
    pub return_flag: bool,
    pub tty_fd: Option<OwnedFd>,
    pub job_table: Vec<JobEntry>,
    current_dir: Option<path::PathBuf>, // the_current_working_directory
    pub completion_functions: HashMap<String, String>,
    pub real_time: TimeSpec, 
    pub user_time: TimeVal, 
    pub sys_time: TimeVal, 
}

fn ignore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigIgn) }
        .expect("sush(fatal): cannot ignore signal");
}

fn restore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigDfl) }
        .expect("sush(fatal): cannot restore signal");
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            data: Data::new(),
            rewritten_history: HashMap::new(),
            history: vec![],
            builtins: HashMap::new(),
            sigint: Arc::new(AtomicBool::new(false)),
            read_stdin: true,
            is_subshell: false,
            source_function_level: 0,
            loop_level: 0,
            break_counter: 0,
            return_flag: false,
            tty_fd: None,
            job_table: vec![],
            current_dir: None,
            completion_functions: HashMap::new(),
            real_time: TimeSpec::new(0, 0),
            user_time: TimeVal::new(0, 0),
            sys_time: TimeVal::new(0, 0),
        };

        core.init_current_directory();
        core.set_initial_parameters();
        core.set_builtins();
        ignore_signal(Signal::SIGPIPE);

        if unistd::isatty(0) == Ok(true) {
            const V: &'static str = env!("CARGO_PKG_VERSION");
            eprintln!("Rusty Bash (a.k.a. Sushi shell), version {}", V);

            core.data.flags += "i";
            core.read_stdin = false;
            core.data.set_param("PS1", "🍣 ");
            core.data.set_param("PS2", "> ");
            let fd = fcntl::fcntl(2, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            core.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }

        let home = core.data.get_param("HOME").to_string();
        core.data.set_param("HISTFILE", &(home + "/.sush_history"));
        core.data.set_param("HISTFILESIZE", "2000");

        core
    }

    fn set_initial_parameters(&mut self) {
        self.data.set_param("$", &process::id().to_string());
        self.data.set_param("BASHPID", &process::id().to_string());
        self.data.set_param("BASH_SUBSHELL", "0");
        self.data.set_param("BASH_VERSION", &(env!("CARGO_PKG_VERSION").to_string() + "-rusty_bash"));
        self.data.set_param("?", "0");
        self.data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()));
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.data.flags.find(flag) != None 
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_pid, status)) => {
                status
            },
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32
            },
            Ok(unsupported) => {
                eprintln!("Unsupported: {:?}", unsupported);
                1
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            },
        };

        if exit_status == 130 {
            self.sigint.store(true, Relaxed);
        }
        self.data.set_layer_param("?", &exit_status.to_string(), 0); //追加
    } 

    fn set_foreground(&self) {
        let fd = match self.tty_fd.as_ref() {
            Some(fd) => fd,
            _        => return,
        };
        let pgid = unistd::getpgid(Some(Pid::from_raw(0)))
                   .expect("sush(fatal): cannot get pgid");

        if unistd::tcgetpgrp(fd) == Ok(pgid) {
            return;
        }

        ignore_signal(Signal::SIGTTOU); //SIGTTOUを無視
        unistd::tcsetpgrp(fd, pgid)
            .expect("sush(fatal): cannot get the terminal");
        restore_signal(Signal::SIGTTOU); //SIGTTOUを受け付け
    }

    fn flip_exit_status(&mut self) {
        match self.data.get_param("?").as_ref() {
            "0" => self.data.set_param("?", "1"),
            _   => self.data.set_param("?", "0"),
        }
    }

    fn show_time(&self) {
           // let user_end_time = time::clock_gettime(ClockId::CLOCK_PROCESS_CPUTIME_ID).unwrap();
            let real_end_time = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();

            let self_usage = nix::sys::resource::getrusage(nix::sys::resource::UsageWho::RUSAGE_SELF).unwrap();
            let children_usage = nix::sys::resource::getrusage(nix::sys::resource::UsageWho::RUSAGE_CHILDREN).unwrap();

            let real_diff = real_end_time - self.real_time;
            eprintln!("\nreal\t{}m{}.{:06}s", real_diff.tv_sec()/60,
                      real_diff.tv_sec()%60, real_diff.tv_nsec()/1000);
            let user_diff = self_usage.user_time() + children_usage.user_time() - self.user_time;
            eprintln!("user\t{}m{}.{:06}s", user_diff.tv_sec()/60,
                      user_diff.tv_sec()%60, user_diff.tv_usec());
            let sys_diff = self_usage.system_time() + children_usage.system_time() - self.sys_time;
            eprintln!("sys \t{}m{}.{:06}s", sys_diff.tv_sec()/60,
                      sys_diff.tv_sec()%60, sys_diff.tv_usec());
    }

    pub fn wait_pipeline(&mut self, pids: Vec<Option<Pid>>,
                         exclamation: bool, time: bool) {
        if pids.len() == 1 && pids[0] == None {
            if time {
                self.show_time();
            }
            if exclamation {
                self.flip_exit_status();
            }
            return;
        }

        let mut pipestatus = vec![];
        for pid in &pids {
            self.wait_process(pid.expect("SUSHI INTERNAL ERROR (no pid)"));
            pipestatus.push(self.data.get_param("?"));
        }

        if time {
            self.show_time();
        }
        self.set_foreground();
        self.data.set_layer_array("PIPESTATUS", &pipestatus, 0);

        if exclamation {
            self.flip_exit_status();
        }
    }

    pub fn run_builtin(&mut self, args: &mut Vec<String>, special_args: &mut Vec<String>) -> bool {
        if args.len() == 0 {
            panic!("SUSH INTERNAL ERROR (no arg for builtins)");
        }

        if self.builtins.contains_key(&args[0]) {
            let func = self.builtins[&args[0]];
            args.append(special_args);
            let status = func(self, args);
            self.data.set_layer_param("?", &status.to_string(), 0);
            return true;
        }

        false
    }

    pub fn exit(&mut self) -> ! {
        self.write_history_to_file();

        let exit_status = match self.data.get_param("?").parse::<i32>() {
            Ok(n)  => n%256,
            Err(_) => {
                eprintln!("sush: exit: {}: numeric argument required", self.data.get_param("?"));
                2
            },
        };
    
        process::exit(exit_status)
    }

    fn set_subshell_parameters(&mut self) {
        let pid = nix::unistd::getpid();
        self.data.set_layer_param("BASHPID", &pid.to_string(), 0);
        match self.data.get_param("BASH_SUBSHELL").parse::<usize>() {
            Ok(num) => self.data.set_layer_param("BASH_SUBSHELL", &(num+1).to_string(), 0),
            Err(_) =>  self.data.set_layer_param("BASH_SUBSHELL", "0", 0),
        };
    }

    pub fn set_pgid(&self, pid: Pid, pgid: Pid) {
        let _ = unistd::setpgid(pid, pgid);
        if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
            self.set_foreground();
        }
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        restore_signal(Signal::SIGINT);
        restore_signal(Signal::SIGPIPE);

        self.is_subshell = true;
        self.set_pgid(pid, pgid);
        self.set_subshell_parameters();
        self.job_table.clear();
    }

    pub fn init_current_directory(&mut self) {
        match env::current_dir() {
            Ok(path) => self.current_dir = Some(path),
            Err(err) => eprintln!("pwd: error retrieving current directory: {:?}", err),
        }
    }

    pub fn get_current_directory(&mut self) -> Option<path::PathBuf> {
        if self.current_dir.is_none() {
            self.init_current_directory();
        }
        self.current_dir.clone()
    }


    pub fn set_current_directory(&mut self, path: &path::PathBuf) -> Result<(), io::Error> {
        let res = env::set_current_dir(path);
        if res.is_ok() {
            self.current_dir = Some(path.clone());
        }
        res
    }
}
