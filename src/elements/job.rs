//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::pipeline::Pipeline;
use crate::{Feeder, ShellCore};
use crate::core::jobtable::JobEntry;
use nix::sys::wait::WaitStatus;
use nix::unistd;
use nix::unistd::{Pid, ForkResult};

#[derive(Debug, Clone)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<String>,
    pub text: String,
}

impl Job {
    pub fn exec(&mut self, core: &mut ShellCore, bg: bool) {
        let pgid = if core.is_subshell { //17〜21行目を追加
            unistd::getpgrp() //自身のPGID
        }else{
            Pid::from_raw(0)
        };

        match bg {
            true  => self.exec_bg(core, pgid),
            false => self.exec_fg(core, pgid),
        }
    }

    fn exec_fg(&mut self, core: &mut ShellCore, pgid: Pid) {
        let mut do_next = true;
        let susp_e_option = core.suspend_e_option;
        for (pipeline, end) in self.pipelines.iter_mut().zip(self.pipeline_ends.iter()) {
            if core.word_eval_error {
                return;
            }

            core.suspend_e_option = susp_e_option || end == "&&" || end == "||";

            if do_next {
                core.jobtable_check_status();
                let (pids, exclamation, time) = pipeline.exec(core, pgid);
                let waitstatuses = core.wait_pipeline(pids.clone(), exclamation, time);

                Self::check_stop(core, &pipeline.text, &pids, &waitstatuses);
            }
            do_next = (core.data.get_param("?") == "0") == (end == "&&");
        }
    }

    fn check_stop(core: &mut ShellCore, text: &str,
                  pids: &Vec<Option<Pid>>, waitstatuses: &Vec<WaitStatus>) {
        if core.is_subshell || pids.len() == 0 || pids[0] == None {
            return;
        }

        for ws in waitstatuses {
            if let WaitStatus::Stopped(_, _) = ws {
                let new_job_id = core.generate_new_job_id();
                let job = JobEntry::new(pids.to_vec(), &waitstatuses, &text, "Stopped", new_job_id); 
                core.job_table_priority.insert(0, new_job_id);
                core.job_table.push(job);
                return;
            }
        }
    }

    fn exec_bg(&mut self, core: &mut ShellCore, pgid: Pid) {
        let backup = match core.tty_fd.as_ref() {
            Some(fd) => Some(fd.try_clone().unwrap()),
            _ => None,
        };
        core.tty_fd = None;

        let pids = if self.pipelines.len() == 1 {
            if self.pipelines[0].commands.len() == 1 {
                self.pipelines[0].commands[0].set_force_fork();
            }
            self.pipelines[0].exec(core, pgid).0
        }else{
            vec![self.exec_fork_bg(core, pgid)]
        };
        eprintln!("{}", &pids[0].unwrap().as_raw());
        let len = pids.len();
        let new_job_id = core.generate_new_job_id();
        core.job_table_priority.insert(0, new_job_id);
        core.job_table.push(JobEntry::new(pids, &vec![ WaitStatus::StillAlive; len ],
                &self.text, "Running", new_job_id));

        core.tty_fd = backup;
    }

    fn exec_fork_bg(&mut self, core: &mut ShellCore, pgid: Pid) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.initialize_as_subshell(Pid::from_raw(0), pgid);
                self.exec(core, false);
                core.exit()
            },
            Ok(ForkResult::Parent { child } ) => {
                core.set_pgid(child, pgid);
                Some(child) 
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    pub fn new() -> Job {
        Job {
            text: String::new(),
            pipeline_ends: vec![],
            pipelines: vec![],
        }
    }

    fn eat_blank_line(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);
        if feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            true
        }else{
            false
        }
    }

    fn eat_pipeline(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        match Pipeline::parse(feeder, core){
            Some(pipeline) => {
                ans.text += &pipeline.text.clone();
                ans.pipelines.push(pipeline);
                true
            },
            None => false,
        }
    }

    fn eat_and_or(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_and_or(core);
        let end = feeder.consume(num);
        ans.pipeline_ends.push(end.clone());
        ans.text += &end;
        num != 0 //記号なしの場合にfalseが返る
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Job> {
        let mut ans = Self::new();
        while Self::eat_blank_line(feeder, &mut ans, core) {} 
        if ! Self::eat_pipeline(feeder, &mut ans, core) {
            return None;
        }

        while Self::eat_and_or(feeder, &mut ans, core) { 
            loop {
                while Self::eat_blank_line(feeder, &mut ans, core) {} 
                if Self::eat_pipeline(feeder, &mut ans, core) {
                    break;  
                }
                if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                    return None;
                }
            }
        }

        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);

        if ans.pipelines.len() > 0 {
//            dbg!("{:?}", &ans); // デバッグ用にansの内容を出力
            Some(ans)
        }else{
            None
        }
    }
}
