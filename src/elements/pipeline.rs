use crate::{core::ShellCore, elem_command::Command, feeder::Feeder};

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub text: String,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for command in self.commands.iter_mut() {
            command.exec(core);
        }
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        if let Some(command) = Command::parse(text, core) {
            return Some(Pipeline {
                text: command.text.clone(),
                commands: vec![command],
            });
        }
        None
    }
}
