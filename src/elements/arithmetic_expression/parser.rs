//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::arithmetic_expression::ArithmeticExpr;
use crate::elements::word::Word;
use super::{Elem, word_manip};

impl ArithmeticExpr {
    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_suffix(feeder: &mut Feeder, ans: &mut Self) -> i64 {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            1
        } else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            -1
        } else{
            0
        }
    }

    fn eat_incdec(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::Increment(1) );
        }else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::Increment(-1) );
        }else {
            return false;
        };
        true
    }

    fn eat_conditional_op(feeder: &mut Feeder,
        ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("?") {
            return false;
        }

        ans.text += &feeder.consume(1);
        let left = Self::parse(feeder, core);
        if left.is_some() {
            ans.text += &left.as_ref().unwrap().text;
        }

        if ! feeder.starts_with(":") {
            ans.elements.push(Elem::ConditionalOp(Box::new(left), Box::new(None)));
            return true;
        }

        ans.text += &feeder.consume(1);
        let right = Self::parse(feeder, core);
        if right.is_some() {
            ans.text += &right.as_ref().unwrap().text;
        }

        ans.elements.push(Elem::ConditionalOp(Box::new(left), Box::new(right)));
        true
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut word = match Word::parse(feeder, core, true) {
            Some(w) => w,
            _       => return false,
        };
        ans.text += &word.text.clone();

        if let Some(w) = word.make_unquoted_word() {
            if word.text.find('\'').is_none() {
                if let Some(n) = word_manip::parse_as_i64(&w) {
                    ans.elements.push( Elem::Integer(n) );
                    return true;
                }
                if let Ok(f) = w.parse::<f64>() {
                    ans.elements.push( Elem::Float(f) );
                    return true;
                }
            }
        }

        Self::eat_blank(feeder, ans, core);

        let suffix = Self::eat_suffix(feeder, ans);
        ans.elements.push( Elem::Word(word, suffix) );
        true
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(Elem::Integer(_)) 
            | Some(Elem::Float(_)) 
            | Some(Elem::Word(_, _)) 
            | Some(Elem::RightParen) => return false,
            _ => {},
        }

        let s = match feeder.scanner_unary_operator(core) {
            0   => return false,
            len => feeder.consume(len),
        };

        ans.text += &s.clone();
        ans.elements.push( Elem::UnaryOp(s) );
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

    fn eat_binary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_binary_operator(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( Elem::BinaryOp(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ArithmeticExpr> {
        let mut ans = ArithmeticExpr::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);

            if feeder.starts_with(":") {
                break;
            }

            if Self::eat_conditional_op(feeder, &mut ans, core) 
            || Self::eat_incdec(feeder, &mut ans) 
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren(feeder, &mut ans)
            || Self::eat_binary_operator(feeder, &mut ans, core)
            || Self::eat_word(feeder, &mut ans, core) { 
                continue;
            }

            if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                break;
            }
        }

        return Some(ans);
    }
}