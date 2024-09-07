//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
enum Elem {
    FileCheckOption(String),
    Word(Word),
    Operand(String),
    RightParen,
    LeftParen,
    Not,  // ! 
    And,  // &&
    Or,  // ||
    Result(bool),
}

pub fn op_order(op: &Elem) -> u8 {
    match op {
        Elem::FileCheckOption(_) => 14,
        _ => 0,
    }
}

pub fn to_operand(w: &Word, core: &mut ShellCore) -> Result<Elem, String> {
    match w.eval_as_value(core) {
        Some(v) => Ok(Elem::Operand(v)),
        None => return Err(format!("{}: wrong substitution", &w.text)),
    }
}

pub fn pop_operand(stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    let n = match stack.pop() {
        Some(Elem::Word(w)) => {
            match to_operand(&w, core) {
                Ok(op) => op,
                Err(e) => return Err(e),
            }
        },
        Some(elem) => elem,
        None       => return Err("no operand".to_string()),
    };
    Ok(n)
}

#[derive(Debug, Clone)]
pub struct TestCommand {
    text: String,
    elements: Vec<Elem>,
    paren_stack: Vec<char>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        let rev_pol = match self.rev_polish() {
            Ok(ans) => ans,
            _ => {
                core.data.set_param("?", "2");
                return;
            },
        };

        let mut stack = vec![];

        for e in rev_pol {
            let result = match e { 
                Elem::Word(_) => {
                    stack.push(e.clone());
                    Ok(())
                },
                Elem::FileCheckOption(ref op)  => Self::unary_operation(&op, &mut stack, core),
                _ => Err( error_message::syntax("TODO")),
            };
    
            if let Err(err_msg) = result {
                core.data.set_param("?", "2");
                return;
            //    return Err(err_msg);
            }
        }
        if stack.len() != 1 { 
            eprintln!("unknown syntax error_message (stack inconsistency)");
            core.data.set_param("?", "2");
            return;
        }   
    
        match stack.pop() {
            Some(Elem::Result(true))  => core.data.set_param("?", "0"),
            Some(Elem::Result(false)) => core.data.set_param("?", "1"),
            _  => {
                eprintln!("unknown syntax error_message");
                core.data.set_param("?", "2");
            },
        }  
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    pub fn rev_polish(&mut self) -> Result<Vec<Elem>, Elem> {
        let mut ans = vec![];
        let mut stack = vec![];
        let mut last = None;
    
        for e in &self.elements {
            let ok = match e {
                Elem::Word(_)    => {ans.push(e.clone()); true},
                Elem::LeftParen  => {stack.push(e.clone()); true},
                Elem::RightParen => Self::rev_polish_paren(&mut stack, &mut ans),
                op               => Self::rev_polish_op(&op, &mut stack, &mut ans),
            };
    
            if !ok {
                return Err(e.clone());
            }
    
            match (last, e) {
                ( Some(Elem::LeftParen), Elem::RightParen ) => return Err(e.clone()),
                _ => {},
            }
    
            last = Some(e.clone());
        }
    
        while stack.len() > 0 {
            ans.push(stack.pop().unwrap());
        }
    
        Ok(ans)
    }

    fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
        let operand = match pop_operand(stack, core) {
            Ok(v)  => v, 
            Err(e) => return Err(e),
        };
        
        match operand {
            Elem::Operand(s)   => Self::unary_calc(op, &s, stack),
            _ => error_message::internal("unknown operand"), 
        }
    }

    fn unary_calc(op: &str, s: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
        match op {
            "-a"  => stack.push( Elem::Result(true) ),
            _  => stack.push( Elem::Result(false) ),
            /*
            "-"  => stack.push( Elem::Integer(-num) ),
            "!"  => stack.push( Elem::Integer(if num == 0 { 1 } else { 0 }) ),
            "~"  => stack.push( Elem::Integer( !num ) ),
            _ => error_message::internal("unknown unary operator"),
            */
        }   
        Ok(())
    }

    fn rev_polish_paren(stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
        loop {
            match stack.last() {
                None => return false,
                Some(Elem::LeftParen) => {
                    stack.pop();
                    return true;
                },
                Some(_) => ans.push(stack.pop().unwrap()),
            }
        }
    }
    
    fn rev_polish_op(elem: &Elem,
                     stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
        loop {
            match stack.last() {
                None | Some(Elem::LeftParen) => {
                    stack.push(elem.clone());
                    break;
                },
                Some(_) => {
                    let last = stack.last().unwrap();
                    if op_order(last) <= op_order(elem) {
                        stack.push(elem.clone());
                        break;
                    }
                    ans.push(stack.pop().unwrap());
                },
            }
        }
    
        true
    }

    fn new() -> TestCommand {
        TestCommand {
            text: String::new(),
            elements: vec![],
            paren_stack: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match Word::parse(feeder, core, false) {
            Some(w) => {
                ans.text += &w.text.clone();
                ans.elements.push(Elem::Word(w));

                true
            },
            _ => false
        }
    }

    fn eat_file_check_option(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_file_check_option(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(Elem::FileCheckOption(opt));

        true
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("(") {
            ans.paren_stack.push( '(' );
            ans.elements.push( Elem::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.paren_stack.pop();
                ans.elements.push( Elem::RightParen );
                ans.text += &feeder.consume(1);
                return true;
            }
        }

        false
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.scanner_blank(core) {
            0 => false,
            n => {
                ans.text += &feeder.consume(n);
                true
            },
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        loop {
            if ! Self::eat_blank(feeder, &mut ans, core) {
                return None;
            }
            if feeder.starts_with("]]") {
                ans.text += &feeder.consume(2);
                command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);

                return match ans.elements.len() > 0 {
                    true  => Some(ans),
                    false => None,
                };
            }

            if Self::eat_file_check_option(feeder, &mut ans, core)
            || Self::eat_paren(feeder, &mut ans) 
            || Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            break;
        }
    
        None
    }
}