// pest. Elegant, efficient grammars
// Copyright (C) 2016  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A `macro` useful for implementing the `Parser` `trait` as a recursive descent parser. It only
/// accepts `grammar!` and `process!` calls that get implemented on `self`.
///
/// # Rule
///
/// It also implements an `enum` called `Rule` that has a value for all
/// [non-silent](macro.grammar!#silent-rules-_) rules, but also for
/// [`any` and `eoi`](macro.grammar!). These `Rule`s are used within `Token`s to specify the type
/// of rule that matched.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate pest;
/// # use pest::prelude::*;
/// # fn main() {
/// impl_rdp! {
///     grammar! {
///         expression = _{ paren ~ expression? }
///         paren      =  { ["("] ~ expression? ~ [")"] }
///     }
/// }
///
/// let mut parser = Rdp::new(StringInput::new("(())((())())()"));
///
/// assert!(parser.expression());
/// assert!(parser.end());
/// # }
/// ```
#[macro_export]
macro_rules! impl_rdp {
    // implement rules
    ( @rules $( $name:ident )* ) => {
        #[allow(dead_code, non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum Rule {
            any,
            eoi,
            $( $name ),*
        }
    };

    // filter out silent rules
    ( @filter [  ] [ $( $rules:tt )* ] ) => {
        impl_rdp!(@rules $( $rules )*);
    };
    ( @filter [ $name:ident = { { $( $_primary:tt )* } $( $ts:tt )* } $( $tail:tt )* ]
      [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* $( $ts )* ] [ $name $( $rules )* ]);
    };
    ( @filter [ $name:ident = @{ { $( $_primary:tt )* } $( $ts:tt )* } $( $tail:tt )* ]
      [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* $( $ts )* ] [ $name $( $rules )* ]);
    };
    ( @filter [ $name:ident = _{ { $( $_primary:tt )* } $( $ts:tt )* } $( $tail:tt )* ]
      [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* $( $ts )* ] [ $name $( $rules )* ]);
    };
    ( @filter [ $name:ident = { $( $_ts:tt )* } $( $tail:tt )* ] [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* ] [ $name $( $rules )* ]);
    };
    ( @filter [ $name:ident = @{ $( $_ts:tt )* } $( $tail:tt )* ] [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* ] [ $name $( $rules )* ]);
    };
    ( @filter [ $name:ident = _{ $( $_ts:tt )* } $( $tail:tt )* ] [ $( $rules:tt )* ] ) => {
        impl_rdp!(@filter [ $( $tail )* ] [ $( $rules )* ]);
    };

    // implement empty whitespace rule
    ( @ws ) => {
        #[allow(dead_code)]
        #[inline]
        pub fn whitespace(&mut self) -> bool {
            false
        }
    };
    ( @ws whitespace = $( $_ts:tt )* ) => ();
    ( @ws $_name:ident = { $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@ws $( $tail )*);
    };
    ( @ws $_name:ident = @{ $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@ws $( $tail )*);
    };
    ( @ws $_name:ident = _{ $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@ws $( $tail )*);
    };

    // implement empty comment rule
    ( @com ) => {
        #[allow(dead_code)]
        #[inline]
        pub fn comment(&mut self) -> bool {
            false
        }
    };
    ( @com comment = $( $_ts:tt )* ) => ();
    ( @com $_name:ident = { $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@com $( $tail )*);
    };
    ( @com $_name:ident = @{ $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@com $( $tail )*);
    };
    ( @com $_name:ident = _{ $( $_ts:tt )* } $( $tail:tt )* ) => {
        impl_rdp!(@com $( $tail )*);
    };

    ( grammar! { $( $ts:tt )* } $( $mac:ident! { $( $rest:tt )* } )* ) => {
        use std::cell::Cell;
        use std::cmp;

        pub struct Rdp<T: Input> {
            input:       T,
            queue:       Vec<Token<Rule>>,
            queue_index: Cell<usize>,
            failures:    Vec<Rule>,
            fail_pos:    usize,
            atomic:      bool,
            comment:     bool,
            eoi_matched: bool
        }

        impl_rdp!(@filter [ $( $ts )* ] []);

        impl<T: Input> Rdp<T> {
            pub fn new(input: T) -> Rdp<T> {
                Rdp {
                    input:       input,
                    queue:       vec![],
                    queue_index: Cell::new(0),
                    failures:    vec![],
                    fail_pos:    0,
                    atomic:      false,
                    comment:     false,
                    eoi_matched: false
                }
            }

            impl_rdp!(@ws $( $ts )*);
            impl_rdp!(@com $( $ts )*);

            #[allow(dead_code)]
            #[inline]
            pub fn any(&mut self) -> bool {
                if self.end() {
                    let pos = self.pos();

                    self.track(Rule::any, pos);

                    false
                } else {
                    let next = self.pos() + 1;
                    self.set_pos(next);

                    true
                }
            }

            #[allow(dead_code)]
            #[inline]
            pub fn eoi(&mut self) -> bool {
                let result = self.end();

                if !result {
                    let pos = self.pos();

                    self.track(Rule::eoi, pos);
                } else {
                    self.eoi_matched = true;
                }

                result
            }

            grammar! {
                $( $ts )*
            }

            $(
                $mac! {
                    $( $rest )*
                }
            )*
        }

        impl<T: Input> Parser for Rdp<T> {
            type Rule = Rule;
            type Token = Token<Rule>;

            #[inline]
            fn match_string(&mut self, string: &str) -> bool {
                self.input.match_string(string)
            }

            #[inline]
            fn match_range(&mut self, left: char, right: char) -> bool {
                self.input.match_range(left, right)
            }

            #[inline]
            fn try<F>(&mut self, revert: bool, rule: F) -> bool
                where F: FnOnce(&mut Self) -> bool {

                let pos = self.input.pos();
                let len = self.queue.len();

                let result = rule(self);

                if revert || !result {
                    self.input.set_pos(pos);
                }

                if !result {
                    self.queue.truncate(len);
                }

                result
            }

            fn prec_climb<F, G>(&mut self, pos: usize, left: usize, min_prec: u8,
                                last_op: Option<(Option<Rule>, u8, bool)>, primary: &mut F,
                                climb: &mut G) -> (Option<(Option<Rule>, u8, bool)>, Option<usize>)
                where F: FnMut(&mut Self) -> bool,
                      G: FnMut(&mut Self) -> Option<(Option<Rule>, u8, bool)> {

                let mut op = if last_op.is_some() {
                    last_op
                } else {
                    climb(self)
                };
                let mut last_right = None;

                while let Some((rule, prec, _)) = op {
                    if prec >= min_prec {
                        let mut new_pos = self.pos();
                        let mut right = self.pos();
                        let queue_pos = self.queue.len();

                        primary(self);

                        if let Some(token) = self.queue.get(queue_pos) {
                            new_pos = token.start;
                            right   = token.end;
                        }

                        op = climb(self);

                        while let Some((_, new_prec, right_assoc)) = op {
                            if new_prec > prec || right_assoc && new_prec == prec {
                                let (new_op, new_lr) = self.prec_climb(queue_pos, new_pos,
                                                                       new_prec, op, primary,
                                                                       climb);

                                op = new_op;
                                last_right = new_lr;
                            } else {
                                break
                            }
                        }

                        if let Some(pos) = last_right {
                            right = cmp::max(pos, right);
                        } else {
                            last_right = Some(right);
                        }

                        if let Some(rule) = rule {
                            let token = Token {
                                rule:  rule,
                                start: left,
                                end:   right
                            };

                            self.queue.insert(pos, token);
                        }
                    } else {
                        return (op, last_right)
                    }
                }

                (op, last_right)
            }

            #[inline]
            fn pos(&self) -> usize {
                self.input.pos()
            }

            #[inline]
            fn set_pos(&mut self, pos: usize) {
                self.input.set_pos(pos);
            }

            #[inline]
            fn end(&self) -> bool {
                self.input.len() == self.input.pos()
            }

            #[inline]
            fn eoi_matched(&self) -> bool {
                self.eoi_matched
            }

            #[inline]
            fn reset(&mut self) {
                self.input.set_pos(0);
                self.queue.clear();
                self.failures.clear();
                self.fail_pos = 0;
            }

            #[inline]
            fn slice_input(&self, start: usize, end: usize) -> &str {
                self.input.slice(start, end)
            }

            #[inline]
            fn queue(&self) -> &Vec<Token<Rule>>{
                &self.queue
            }

            #[inline]
            fn queue_index(&self) -> usize {
                self.queue_index.get()
            }

            #[inline]
            fn inc_queue_index(&self) {
                self.queue_index.set(self.queue_index.get() + 1);
            }

            #[inline]
            fn set_queue_index(&self, index: usize) {
                self.queue_index.set(index);
            }

            #[inline]
            fn queue_mut(&mut self) -> &mut Vec<Token<Rule>>{
                &mut self.queue
            }

            #[inline]
            fn skip_ws(&mut self) {
                if self.atomic {
                    return
                }

                loop {
                    if !self.whitespace() {
                        break
                    }
                }
            }

            fn skip_com(&mut self) {
                if self.atomic {
                    return
                }

                if !self.comment {
                    self.comment = true;

                    loop {
                        if !self.comment() {
                            break
                        }
                    }

                    self.comment = false;
                }
            }

            fn is_atomic(&self) -> bool {
                self.atomic
            }

            fn set_atomic(&mut self, value: bool) {
                self.atomic = value;
            }

            fn track(&mut self, failed: Rule, pos: usize) {
                if self.atomic {
                    return
                }

                if self.failures.is_empty() {
                    self.failures.push(failed);

                    self.fail_pos = pos;
                } else {
                    if pos == self.fail_pos {
                        self.failures.push(failed);
                    } else if pos > self.fail_pos {
                        self.failures.clear();
                        self.failures.push(failed);

                        self.fail_pos = pos;
                    }
                }
            }

            fn tracked_len(&self) -> usize {
                self.failures.len()
            }

            fn expected(&mut self) -> (Vec<Rule>, usize) {
                self.failures.sort();
                self.failures.dedup();

                (self.failures.iter().cloned().collect(), self.fail_pos)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::super::super::prelude::*;

    impl_rdp! {
        grammar! {
            expression = _{ paren ~ expression? }
            paren = { ["("] ~ expression? ~ [")"] }
            zero = { ["a"]* }
            one = { ["a"]+ }
            comment = _{ ["//"] ~ (!["\n"] ~ any)* ~ ["\n"] }
            whitespace = _{ [" "] }
        }
    }

    #[test]
    fn match_string() {
        let input = StringInput::new("asdasdf");
        let mut parser = Rdp::new(input);

        assert!(parser.match_string("asd"));
        assert!(parser.match_string("asdf"));
        assert!(parser.match_string(""));
        assert!(!parser.match_string("a"));
    }

    #[test]
    fn try() {
        let input = StringInput::new("asdasdf");
        let mut parser = Rdp::new(input);

        assert!(parser.match_string("asd"));

        assert!(!parser.try(false, |parser| {
            parser.match_string("as") && parser.match_string("dd")
        }));

        assert!(parser.try(false, |parser| {
            parser.match_string("as") && parser.match_string("df")
        }));
    }

    #[test]
    fn end() {
        let input = StringInput::new("asdasdf");
        let mut parser = Rdp::new(input);

        assert!(parser.match_string("asdasdf"));
        assert!(parser.end());
    }

    #[test]
    fn reset() {
        let input = StringInput::new("asdasdf");
        let mut parser = Rdp::new(input);

        assert!(parser.match_string("asdasdf"));

        parser.reset();

        assert!(parser.match_string("asdasdf"));
    }

    #[test]
    fn whitespace_seq() {
        let mut parser = Rdp::new(StringInput::new("  (  ( ))(( () )() )() "));

        assert!(parser.expression());
        assert!(!parser.end());

        let queue = vec![
            Token { rule: Rule::paren, start: 2, end: 9 },
            Token { rule: Rule::paren, start: 5, end: 8 },
            Token { rule: Rule::paren, start: 9, end: 20 },
            Token { rule: Rule::paren, start: 10, end: 16 },
            Token { rule: Rule::paren, start: 12, end: 14 },
            Token { rule: Rule::paren, start: 16, end: 18 },
            Token { rule: Rule::paren, start: 20, end: 22 }
        ];

        assert_eq!(parser.queue(), &queue);
    }

    #[test]
    fn whitespace_zero() {
        let mut parser = Rdp::new(StringInput::new("  a a aa aaaa a  "));

        assert!(parser.zero());
        assert!(!parser.end());

        let queue = vec![
            Token { rule: Rule::zero, start: 2, end: 15 }
        ];

        assert_eq!(parser.queue(), &queue);
    }

    #[test]
    fn whitespace_one() {
        let mut parser = Rdp::new(StringInput::new("  a a aa aaaa a  "));

        assert!(parser.one());
        assert!(!parser.end());

        let queue = vec![
            Token { rule: Rule::one, start: 2, end: 15 }
        ];

        assert_eq!(parser.queue(), &queue);
    }

    #[test]
    fn comment() {
        let mut parser = Rdp::new(StringInput::new("// hi\n(())"));

        assert!(parser.expression());
        assert!(parser.end());

        let queue = vec![
            Token { rule: Rule::paren, start: 6, end: 10 },
            Token { rule: Rule::paren, start: 7, end: 9 }
        ];

        assert_eq!(parser.queue(), &queue);
    }

    #[test]
    fn comment_whitespace() {
        let mut parser = Rdp::new(StringInput::new("   // hi\n  (())"));

        assert!(parser.expression());
        assert!(parser.end());

        let queue = vec![
            Token { rule: Rule::paren, start: 11, end: 15 },
            Token { rule: Rule::paren, start: 12, end: 14 }
        ];

        assert_eq!(parser.queue(), &queue);
    }
}
