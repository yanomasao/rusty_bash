use crate::{core::ShellCore, feeder::Feeder};

use super::job::Job;

pub struct Script {
    pub jobs: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for job in self.jobs.iter_mut() {
            job.exec(core);
        }
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if let Some(job) = Job::parse(text, core) {
            return Some(Script {
                text: job.text.clone(),
                jobs: vec![job],
            });
        }
        None
    }
}
