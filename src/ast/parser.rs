use std::mem;

use crate::ast::lexer::{Token, TokenKind};
use crate::ast::{Alternative, Char, Disjunction, PatternKind, AST};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    ast: &'a mut AST,
    current: usize,
}

// Regex: /ab|cd/
// 0.
//    parent = None
//    node = None
// 1. a (char)
//    parent = None
//    node = { char }
// 2. b (char)
//    parent = None
//    node = { alternative: { char, char } }
// 3. | (bar)
//    parent = { disjunction: { left: { alternative: { char, char } }, right: {} } }
//    node = None
// 4. c (char)
//    parent = { disjunction: { left: { alternative: { char, char } }, right: { char } } }
//    node = { char }
// 5. d (char)
//    parent = { disjunction: { left: { alternative: { char, char } }, right: alternative: { char, char } } }
//    node = { alternative: { char, char } }

#[derive(Debug)]
struct NodeContext {
    parent: Option<Box<PatternKind>>,
    node: Option<Box<PatternKind>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, ast: &'a mut AST) -> Self {
        Self {
            tokens,
            ast,
            current: 0,
        }
    }

    // fn add_char(&mut self, ctx: &mut NodeContext) {
    //     let c = self.parse_char();
    //     let node = ctx.node.unwrap();
    //     let x = *node;
    //     match x {
    //         prev_c @ PatternKind::Char(_) => {
    //             let expressions = vec![Box::new(prev_c), Box::new(c)];
    //             let alt = PatternKind::Alternative(Alternative { expressions });
    //             ctx.node = Some(Box::new(alt));
    //             // ctx.parent = mem::replace(dest, src)
    //         }
    //         PatternKind::Alternative(mut alt) => {
    //             alt.expressions.push(Box::new(c));
    //         }
    //         // PatternKind::Disjunction(mut dist) => {
    //         //     dist.right = Some(Box::new(c));
    //         // }
    //         _ => unreachable!(),
    //     }
    // }
    //
    pub fn parse(&mut self) {
        todo!()
        //     let mut prev: Option<PatternKind> = None;
        //     let mut ctx = NodeContext {
        //         node: None,
        //         parent: None,
        //     };
        //
        //     while let Some(t) = self.next_item(&ctx) {
        //         dbg!(&t);
        //         // prev = Some(t);
        //     }
        //
        //     // if let Some(t) = prev {
        //     //     self.ast.body = t;
        //     // }
        //     self.ast.body = *ctx.parent;
    }
    //
    // pub fn next_item(&mut self, ctx: &mut NodeContext) -> Option<PatternKind> {
    //     if self.is_at_end() {
    //         return None;
    //     }
    //
    //     self.parse_item(&ctx);
    //     Some()
    // }
    //
    // pub fn is_at_end(&self) -> bool {
    //     self.current().kind == TokenKind::EOF
    // }
    //
    // // /a|bc/
    // fn parse_item(&mut self, node: &mut Option<PatternKind>, parent: &mut Option<PatternKind>) {
    //     match &self.current().kind {
    //         TokenKind::Char => {
    //             // if let Some(nc) = self.peek(1) {
    //             //     if nc.kind == TokenKind::Char {
    //             //         return self.parse_alternative();
    //             //     } else if nc.kind == TokenKind::Bar {
    //             //         return self.parse_disjunction();
    //             //     }
    //             // }
    //             //
    //             // self.parse_char()
    //             self.add_char(ctx);
    //         }
    //         _ => unreachable!(),
    //         // TokenKind::Bar =>
    //     }
    // }

    fn parse_char(&mut self) -> PatternKind {
        let t = self.consume_and_check(TokenKind::Char).to_owned();

        PatternKind::Char(Char {
            value: t.span.literal.chars().nth(0).unwrap(),
            token: t,
        })
    }

    fn parse_alternative(&mut self) -> PatternKind {
        let mut expressions = vec![];

        while self.current().kind == TokenKind::Char {
            // if let PatternKind::Char(c) = self.parse_char() {
            //     expressions.push(c);
            // }

            // let t = self.consume();
            // let c = Char {
            //     value: t.span.literal.chars().nth(0).unwrap(),
            //     token: t.clone(),
            // };
            // expressions.push(Box::new(PatternKind::Char(c)));
            let c = self.parse_char();
            expressions.push(Box::new(c));
        }

        PatternKind::Alternative(Alternative { expressions })
    }

    fn parse_disjunction(&mut self) -> PatternKind {
        // if let PatternKind::Char(c) = self.parse_char() {
        //     expressions.push(c);
        // }

        // let t = self.consume();
        // let left = Char {
        //     value: t.span.literal.chars().nth(0).unwrap(),
        //     token: t.clone(),
        // };
        let left = self.parse_char();

        self.consume(); // discard bar token

        // let t = self.consume();
        // let right = Char {
        //     value: t.span.literal.chars().nth(0).unwrap(),
        //     token: t.clone(),
        // };
        let right = self.parse_char();
        //
        // let alt = PatternKind::Disjunction(Disjunction {
        //     left: Box::new(PatternKind::Char(left)),
        //     right: Box::new(PatternKind::Char(right)),
        // });

        let dist = PatternKind::Disjunction(Disjunction {
            left: Box::new(left),
            right: Some(Box::new(right)),
        });

        dist
    }

    fn peek(&self, offset: isize) -> Option<&Token> {
        let index = (self.current as isize + offset) as usize;
        if index >= self.tokens.len() {
            return None;
        }

        self.tokens.get(index)
    }

    fn current(&self) -> &Token {
        self.peek(0).unwrap()
    }

    fn consume(&mut self) -> &Token {
        self.current += 1;
        self.peek(-1).unwrap()
    }

    fn consume_and_check(&mut self, kind: TokenKind) -> &Token {
        let token = self.consume();
        if token.kind != kind {
            panic!("invalid token {}", kind);
        }
        token
    }

    fn add_disjunction(opt: &mut Option<Box<PatternKind>>) {
        // *opt = Some(String::from("ok"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::lexer;

    #[test]
    fn test_parser() {
        let ast = AST { body: None };

        let mut ctx = NodeContext {
            node: None,
            parent: None,
        };

        let c = Box::new(PatternKind::Char(Char {
            value: 'a',
            token: Token {
                kind: lexer::TokenKind::Char,
                span: lexer::TextSpan {
                    start: 0,
                    end: 1,
                    literal: "a".to_string(),
                },
            },
        }));

        ctx.node = Some(c);
        dbg!(&ctx);

        let c = Box::new(PatternKind::Char(Char {
            value: 'b',
            token: Token {
                kind: lexer::TokenKind::Char,
                span: lexer::TextSpan {
                    start: 0,
                    end: 1,
                    literal: "b".to_string(),
                },
            },
        }));

        let prev_c = ctx.node.take().unwrap();
        let expressions = vec![prev_c, c];

        let alt = PatternKind::Alternative(Alternative { expressions });
        ctx.node = Some(Box::new(alt));

        dbg!(&ctx);

        let left = ctx.node.take().unwrap();

        let dist = PatternKind::Disjunction(Disjunction { left, right: None });
        ctx.node = Some(Box::new(dist));

        dbg!(&ctx);

        let c = Box::new(PatternKind::Char(Char {
            value: 'b',
            token: Token {
                kind: lexer::TokenKind::Char,
                span: lexer::TextSpan {
                    start: 0,
                    end: 1,
                    literal: "b".to_string(),
                },
            },
        }));

        // ctx.node = match ctx.node {
        //     Some(dd) => match *dd {
        //         PatternKind::Disjunction(mut d) => {
        //             d.right = Some(c);
        //             // Some(Box::new(PatternKind::Disjunction(d)))
        //         }
        //         _ => unreachable!(),
        //     },
        //     None => None,
        // };

        // let node = ctx.node.take().unwrap();
        // // let n = *node;
        // match *node {
        //     PatternKind::Disjunction(mut d) => {
        //         d.right = Some(c);
        //         ctx.node = d.right;
        //         // Some(Box::new(PatternKind::Disjunction(d)))
        //         // Some(dd)
        //     }
        //     _ => unreachable!(),
        // };

        dbg!(&ctx);
        dbg!(&ctx.node);

        // assert_eq!(
        //     ast,
        //     AST {
        //         body: None,
        //         //     body: PatternKind::Char(Char {
        //         //         value: 'a',
        //         //         token: Token {
        //         //             kind: lexer::TokenKind::Char,
        //         //             span: lexer::TextSpan {
        //         //                 start: 0,
        //         //                 end: 1,
        //         //                 literal: "a".to_string(),
        //         //             },
        //         //         },
        //         //     }),
        //         // }
        //     }
        // );
    }
}
