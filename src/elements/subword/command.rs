//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: ParenCommand,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn substitute(&mut self, core: &mut ShellCore) -> bool {true}
    fn get_type(&self) -> SubwordType { SubwordType::CommandSubstitution }
}

impl CommandSubstitution {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$(") {
            return None;
        }
        let mut text = feeder.consume(1);

        if let Some(pc) = ParenCommand::parse(feeder, core) {
            text += &pc.get_text();
            Some( CommandSubstitution { text: text, command: pc } )
        }else{
            None
        }
    }
}