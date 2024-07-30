//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;

use crate::{ShellCore, Feeder};
use self::calculator::calculate;
use super::word::Word;

#[derive(Debug, Clone)]
enum CalcElement {
    UnaryOp(String),
    BinaryOp(String),
    Num(i64),
//    Name(String),
    Name(String, i32),
    Word(Word),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    elements: Vec<CalcElement>,
    paren_stack: Vec<char>,
}

impl Calc {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        let es = match self.evaluate_elems(core) {
            Ok(data)     => data, 
            Err(err_msg) => {
                eprintln!("sush: {}", err_msg);
                return None;
            },
        };

        match calculate(&es) {
            Ok(ans)  => Some(ans),
            Err(msg) => {
                eprintln!("sush: {}: {}", &self.text, msg);
                None
            },
        }
    }

    fn evaluate_elems(&self, core: &mut ShellCore) -> Result<Vec<CalcElement>, String> {
        let mut ans = vec![];

        for e in &self.elements {
            match e {
                /*
                CalcElement::Name(s) => {
                    let val = core.data.get_param(s);
                    match self.value_to_num(&val, "", 0) {
                        Ok(e)        => ans.push(e),
                        Err(err_msg) => return Err(err_msg), 
                    }
                },*/
                CalcElement::Name(s, i) => {
                    let val = core.data.get_param(s);
                    match self.value_to_num(&val, "", *i) {
                        Ok(e)        => ans.push(e),
                        Err(err_msg) => return Err(err_msg), 
                    }

                    core.data.set_param(&s, &(val.parse::<i32>().unwrap_or(0) + i).to_string());
                },
                CalcElement::Word(w) => {
                    let val = match w.eval_as_value(core) {
                        Some(v) => v, 
                        None => return Err(format!("{}: wrong substitution", &self.text)),
                    };

                    match self.value_to_num(&val, &w.text, 0) {
                        Ok(e)        => ans.push(e),
                        Err(err_msg) => return Err(err_msg), 
                    }
                },
                _ => ans.push(e.clone()),
            }
        }

        Ok(ans)
    }

    fn value_to_num(&self, val: &String, text: &str, inc: i32) -> Result<CalcElement, String> {
        if text.find('\'').is_some() {
            Ok( CalcElement::Name("\'".to_owned() + val + "\'", 0) )
        }else if val == "" {
            Ok( CalcElement::Num(0) )
        }else if let Ok(n) = val.parse::<i64>() {
            Ok( CalcElement::Num(n) )
        }else if inc != 0 {
            Ok( CalcElement::Num(0) )
        }else {
            Err(format!("{0}: syntax error: operand expected (error token is \"{0}\")", &val))
        }
    }

    pub fn new() -> Calc {
        Calc {
            text: String::new(),
            elements: vec![],
            paren_stack: vec![],
        }
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_integer(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(CalcElement::Num(_)) => return false,
            _ => {},
        }

        let len = feeder.scanner_nonnegative_integer(core);
        if len == 0 {
            return false;
        }

        let n = match feeder.refer(len).parse::<i64>() {
            Ok(n)  => n, 
            Err(_) => return false,
        };

        ans.inc_dec_to_unarys();
        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::Num(n) );
        true
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s;
        Self::eat_blank(feeder, ans, core);

        if feeder.starts_with("++") {
            ans.elements.push( CalcElement::Name(s.clone(), 1) );
            ans.text += &feeder.consume(2);
        } else if feeder.starts_with("--") {
            ans.elements.push( CalcElement::Name(s.clone(), -1) );
            ans.text += &feeder.consume(2);
        } else{
            ans.elements.push( CalcElement::Name(s.clone(), 0) );
        }

        true
    }

    fn eat_incdec(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CalcElement::UnaryOp("++".to_string()) );
        }else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CalcElement::UnaryOp("--".to_string()) );
        }else {
            return false;
        };
        true
    }

    fn inc_dec_to_unarys(&mut self) {
        if let Some(CalcElement::UnaryOp(op)) = self.elements.last() {
            let pm = match op.as_str() {
                "++" => "+",
                "--" => "-",
                _    => return, 
            }.to_string();

            self.elements.pop();

            match self.elements.last() {
                None |
                Some(CalcElement::UnaryOp(_)) |
                Some(CalcElement::BinaryOp(_)) |
                Some(CalcElement::LeftParen) 
                   => self.elements.push(CalcElement::UnaryOp(pm.clone())),
                _  => self.elements.push(CalcElement::BinaryOp(pm.clone())),
            }
            self.elements.push(CalcElement::UnaryOp(pm));
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match Word::parse(feeder, core) {
            Some(w) => {
                ans.text += &w.text;
                ans.elements.push( CalcElement::Word(w) );
                true
            },
            _ => false,
        }
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(CalcElement::Num(_)) => return false,
            Some(CalcElement::Name(_, _)) => return false,
            Some(CalcElement::Word(_)) => return false,
            _ => {},
        }

        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        let s = if feeder.starts_with("+") || feeder.starts_with("-") {
            feeder.consume(1)
        }else{
            return false
        };

        ans.inc_dec_to_unarys();
        ans.text += &s.clone();
        ans.elements.push( CalcElement::UnaryOp(s) );
        true
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("(") {
            ans.inc_dec_to_unarys();
            ans.paren_stack.push( '(' );
            ans.elements.push( CalcElement::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.inc_dec_to_unarys();
                ans.paren_stack.pop();
                ans.elements.push( CalcElement::RightParen );
                ans.text += &feeder.consume(1);
                return true;
            }
        }

        false
    }

    fn eat_binary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        ans.inc_dec_to_unarys();
        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::BinaryOp(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if Self::eat_name(feeder, &mut ans, core) 
            || Self::eat_incdec(feeder, &mut ans) 
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren(feeder, &mut ans)
            || Self::eat_binary_operator(feeder, &mut ans, core)
            || Self::eat_integer(feeder, &mut ans, core) 
            || Self::eat_word(feeder, &mut ans, core) { 
                continue;
            }

            if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                break;
            }
        }

        match feeder.starts_with("))") {
            true  => Some(ans),
            false => None,
        }
    }
}
