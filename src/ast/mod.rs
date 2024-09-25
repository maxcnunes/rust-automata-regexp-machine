use self::lexer::Token;

pub mod lexer;
pub mod parser;

#[derive(Debug, PartialEq)]
pub struct AST {
    // type: "RegExp",
    body: Option<PatternKind>, // flags: String,
}

impl AST {
    pub fn new() -> Self {
        Self { body: None }
    }

    // fn add_char(&mut self, node: &mut Option<PatternKind>, parent: &mut Option<PatternKind>) {
    //     let c = self.parse_char();
    //     match *node {
    //         Some(pc @ PatternKind::Char(_)) => {
    //             let expressions = vec![Box::new(pc), Box::new(c)];
    //             let alt = PatternKind::Alternative(Alternative { expressions });
    //             *node = Some(alt);
    //         }
    //         None => unreachable!(),
    //         _ => unreachable!(),
    //     }
    // }
}

#[derive(Debug, PartialEq)]
pub enum PatternKind {
    Char(Char),
    Alternative(Alternative),
    Disjunction(Disjunction),
    Empty,
}

#[derive(Debug, PartialEq)]
pub struct Char {
    value: char,
    token: Token,
}

#[derive(Debug, PartialEq)]
pub struct Alternative {
    expressions: Vec<Box<PatternKind>>,
}

#[derive(Debug, PartialEq)]
pub struct Disjunction {
    left: Box<PatternKind>,
    right: Option<Box<PatternKind>>,
}
//
// #[derive(Debug, PartialEq)]
// pub enum DisjunctionItem {
//     Char(Char),
//     Alternative(Alternative),
// }

pub fn parse(input: &str) -> AST {
    let tokens = lexer::tokens(&input);
    let mut ast = AST::new();
    let mut p = parser::Parser::new(tokens, &mut ast);
    p.parse();
    ast
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn parse_single_char() {
//         // {
//         //   "type": "RegExp",
//         //   "body": {
//         //     "type": "Char",
//         //     "value": "a",
//         //     "kind": "simple",
//         //     "symbol": "a",
//         //     "codePoint": 97
//         //   },
//         //   "flags": ""
//         // }
//         let input = "a";
//         let ast = parse(input);
//         dbg!(&ast);
//
//         assert_eq!(
//             ast,
//             AST {
//                 body: PatternKind::Char(Char {
//                     value: 'a',
//                     token: Token {
//                         kind: lexer::TokenKind::Char,
//                         span: lexer::TextSpan {
//                             start: 0,
//                             end: 1,
//                             literal: "a".to_string(),
//                         },
//                     },
//                 }),
//             }
//         );
//     }
//
//     #[test]
//     fn parse_multiple_chars_alternative() {
//         // {
//         //   "type": "RegExp",
//         //   "body": {
//         //     "type": "Alternative",
//         //     "expressions": [
//         //       {
//         //         "type": "Char",
//         //         "value": "a",
//         //         "kind": "simple",
//         //         "symbol": "a",
//         //         "codePoint": 97
//         //       },
//         //       {
//         //         "type": "Char",
//         //         "value": "b",
//         //         "kind": "simple",
//         //         "symbol": "b",
//         //         "codePoint": 98
//         //       },
//         //     ]
//         //   },
//         //   "flags": ""
//         // }
//         let input = "ab";
//         let ast = parse(input);
//         dbg!(&ast);
//
//         assert_eq!(
//             ast.body,
//             PatternKind::Alternative(Alternative {
//                 expressions: vec![
//                     Box::new(PatternKind::Char(Char {
//                         value: 'a',
//                         token: Token {
//                             kind: lexer::TokenKind::Char,
//                             span: lexer::TextSpan {
//                                 start: 0,
//                                 end: 1,
//                                 literal: "a".to_string(),
//                             },
//                         },
//                     })),
//                     Box::new(PatternKind::Char(Char {
//                         value: 'b',
//                         token: Token {
//                             kind: lexer::TokenKind::Char,
//                             span: lexer::TextSpan {
//                                 start: 1,
//                                 end: 2,
//                                 literal: "b".to_string(),
//                             },
//                         },
//                     })),
//                 ],
//             })
//         );
//     }
//
//     #[test]
//     fn parse_disjunction_left_char_right_char() {
//         // {
//         //   "type": "RegExp",
//         //   "body": {
//         //     "type": "Disjunction",
//         //     "left": {
//         //       "type": "Char",
//         //       "value": "a",
//         //       "kind": "simple",
//         //       "symbol": "a",
//         //       "codePoint": 97
//         //     },
//         //     "right": {
//         //       "type": "Char",
//         //       "value": "b",
//         //       "kind": "simple",
//         //       "symbol": "b",
//         //       "codePoint": 98
//         //     }
//         //   },
//         //   "flags": ""
//         // }
//         let input = "a|b";
//         let ast = parse(input);
//         dbg!(&ast);
//
//         assert_eq!(
//             ast.body,
//             PatternKind::Disjunction(Disjunction {
//                 left: Box::new(PatternKind::Char(Char {
//                     value: 'a',
//                     token: Token {
//                         kind: lexer::TokenKind::Char,
//                         span: lexer::TextSpan {
//                             start: 0,
//                             end: 1,
//                             literal: "a".to_string(),
//                         },
//                     },
//                 })),
//                 right: Box::new(PatternKind::Char(Char {
//                     value: 'b',
//                     token: Token {
//                         kind: lexer::TokenKind::Char,
//                         span: lexer::TextSpan {
//                             start: 2,
//                             end: 3,
//                             literal: "b".to_string(),
//                         },
//                     },
//                 })),
//             },)
//         );
//     }
//
//     #[test]
//     fn parse_disjunction_left_char_right_alternative() {
//         // {
//         //   "type": "RegExp",
//         //   "body": {
//         //     "type": "Disjunction",
//         //     "left": {
//         //       "type": "Char",
//         //       "value": "a",
//         //       "kind": "simple",
//         //       "symbol": "a",
//         //       "codePoint": 97
//         //     },
//         //     "right": {
//         //       "type": "Alternative",
//         //       "expressions": [
//         //         {
//         //           "type": "Char",
//         //           "value": "b",
//         //           "kind": "simple",
//         //           "symbol": "b",
//         //           "codePoint": 98
//         //         },
//         //         {
//         //           "type": "Char",
//         //           "value": "c",
//         //           "kind": "simple",
//         //           "symbol": "c",
//         //           "codePoint": 99
//         //         }
//         //       ]
//         //     }
//         //   },
//         //   "flags": ""
//         // }
//         let input = "a|bc";
//         let ast = parse(input);
//         dbg!(&ast);
//
//         assert_eq!(
//             ast.body,
//             PatternKind::Disjunction(Disjunction {
//                 left: Box::new(PatternKind::Char(Char {
//                     value: 'a',
//                     token: Token {
//                         kind: lexer::TokenKind::Char,
//                         span: lexer::TextSpan {
//                             start: 0,
//                             end: 1,
//                             literal: "a".to_string(),
//                         },
//                     },
//                 })),
//                 right: Box::new(PatternKind::Alternative(Alternative {
//                     expressions: vec![
//                         Box::new(PatternKind::Char(Char {
//                             value: 'b',
//                             token: Token {
//                                 kind: lexer::TokenKind::Char,
//                                 span: lexer::TextSpan {
//                                     start: 1,
//                                     end: 2,
//                                     literal: "b".to_string(),
//                                 },
//                             },
//                         })),
//                         Box::new(PatternKind::Char(Char {
//                             value: 'c',
//                             token: Token {
//                                 kind: lexer::TokenKind::Char,
//                                 span: lexer::TextSpan {
//                                     start: 2,
//                                     end: 3,
//                                     literal: "c".to_string(),
//                                 },
//                             },
//                         })),
//                     ],
//                 }))
//             })
//         );
//     }
// }
