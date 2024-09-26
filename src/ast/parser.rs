// This module was heavily based and copied from https://github.com/rust-lang/regex/blob/master/regex-syntax/src/ast/parse.rs

use crate::ast;
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};

type Result<T> = core::result::Result<T, ast::Error>;

/// Primitive is an expression with no sub-expressions.
enum Primitive {
    Literal(ast::Literal),
}

impl Primitive {
    fn into_ast(self) -> ast::AST {
        match self {
            Primitive::Literal(lit) => ast::AST::literal(lit),
        }
    }
}

/// GroupState represents a single stack frame while parsing nested groups
/// and alternations. Each frame records the state up to an opening parenthesis
/// or an alternating `|`.
enum GroupState {
    /// This state is pushed whenever an opening group is found.
    // Group {
    //     /// The concatenation immediately precending the opening group.
    //     concat: ast::Concat,
    // },
    Alternation(ast::Alternation),
}

/// A regular expression parser.
pub struct Parser {
    /// The current position of the parser.
    pos: Cell<ast::Position>,
    /// A stack of grouped sub-expressions, including alternations.
    stack_group: RefCell<Vec<GroupState>>,
}

/// ParserI is the internal parser implementation.
struct ParserI<'s, P> {
    /// The parser state/configuration.
    parser: P,
    /// The full regular expression provided by the user.
    pattern: &'s str,
}

impl Parser {
    /// Create a new parser with default configuration.
    pub fn new() -> Parser {
        Parser {
            pos: Cell::new(ast::Position { offset: 0 }),
            stack_group: RefCell::new(vec![]),
        }
    }

    pub fn parse(&mut self, pattern: &str) -> Result<ast::AST> {
        ParserI::new(self, pattern).parse()
    }
}

impl<'s, P: Borrow<Parser>> ParserI<'s, P> {
    /// Build an internal parser from a parser configuration and  pattern.
    fn new(parser: P, pattern: &'s str) -> ParserI<'s, P> {
        ParserI { parser, pattern }
    }

    /// Return a reference of the parser state.
    fn parser(&self) -> &Parser {
        self.parser.borrow()
    }

    /// Return a reference to the pattern being parsed.
    fn pattern(&self) -> &str {
        self.pattern
    }

    /// Return true if the next call to bump would return false.
    fn is_eof(&self) -> bool {
        self.offset() == self.pattern().len()
    }

    /// Return the current offset of the parser.
    fn offset(&self) -> usize {
        self.parser().pos.get().offset
    }

    /// Return the current position of the parser.
    fn pos(&self) -> ast::Position {
        self.parser().pos.get()
    }

    /// Create a span at the current position of the parser.
    fn span(&self) -> ast::Span {
        ast::Span::splat(self.pos())
    }

    /// Create a span that converts the current character.
    fn span_char(&self) -> ast::Span {
        let next = ast::Position {
            offset: self.offset().checked_add(self.char().len_utf8()).unwrap(),
            // line: self.line(),
            // column: self.column().checked_add(1).unwrap(),
        };

        // if self.char() == '\n' {
        //     next.line += 1;
        //     next.column = 1;
        // }

        ast::Span::new(self.pos(), next)
    }

    /// Return character at the current position of the parser.
    fn char(&self) -> char {
        self.char_at(self.offset())
    }

    /// Return a character at the given position.
    fn char_at(&self, i: usize) -> char {
        self.pattern()[i..]
            .chars()
            .next()
            .unwrap_or_else(|| panic!("expected char at offset {}", i))
    }

    // Parse the regular expression into an abstract syntax tree.
    fn parse(&self) -> Result<ast::AST> {
        assert_eq!(self.offset(), 0, "parser can only be used once");

        let mut concat = ast::Concat {
            span: self.span(),
            asts: vec![],
        };

        loop {
            // self.bump_space();
            if self.is_eof() {
                break;
            }

            match self.char() {
                '|' => concat = self.push_alternate(concat)?,
                _ => concat.asts.push(self.parse_primitive()?.into_ast()),
            }
        }

        let ast = self.pop_group_end(concat)?;
        Ok(ast)
    }

    /// Bump the parser to the next Unicode scalar value.
    fn bump(&self) -> bool {
        if self.is_eof() {
            return false;
        }

        let ast::Position {
            mut offset, /*, mut line, mut column*/
        } = self.pos();
        // if self.char() == '\n' {
        //     line = line.checked_add(1).unwrap();
        //     column = 1;
        // } else {
        //     column = column.checked_add(1).unwrap();
        // }

        offset += self.char().len_utf8();
        self.parser().pos.set(ast::Position { offset });
        self.pattern()[self.offset()..].chars().next().is_some()
    }

    /// Parse a primitive AST. e.g., a literal, non-set character class, or assertion.
    fn parse_primitive(&self) -> Result<Primitive> {
        match self.char() {
            c => {
                let ast = Primitive::Literal(ast::Literal {
                    span: self.span_char(),
                    kind: ast::LiteralKind::Verbatim,
                    c,
                });
                self.bump();
                Ok(ast)
            }
        }
    }

    /// Parse and push a single alternation on the parser's internal stack.
    /// If the top of the stack already has an alternation, then add to that
    /// instead of pushing a new one.
    fn push_alternate(&self, mut concat: ast::Concat) -> Result<ast::Concat> {
        assert_eq!(self.char(), '|');
        concat.span.end = self.pos();
        self.push_or_add_alternation(concat);
        self.bump();
        Ok(ast::Concat {
            span: self.span(),
            asts: vec![],
        })
    }

    /// Pushes or adds the given branch of an alernation to the parser's
    /// internal stack of state.
    fn push_or_add_alternation(&self, concat: ast::Concat) {
        let mut stack = self.parser().stack_group.borrow_mut();
        if let Some(&mut GroupState::Alternation(ref mut alts)) = stack.last_mut() {
            alts.asts.push(concat.into_ast());
            return;
        }

        stack.push(GroupState::Alternation(ast::Alternation {
            span: ast::Span::new(concat.span.start, self.pos()),
            asts: vec![concat.into_ast()],
        }));
    }

    /// Pop a group AST from the parser's internal stack  and set the group's
    /// AST to the given concatenation. Return the concatenation containing
    /// the group.
    fn pop_group_end(&self, mut concat: ast::Concat) -> Result<ast::AST> {
        concat.span.end = self.pos();

        let mut stack = self.parser().stack_group.borrow_mut();
        let ast = match stack.pop() {
            None => Ok(concat.into_ast()),
            Some(GroupState::Alternation(mut alt)) => {
                alt.span.end = self.pos();
                alt.asts.push(concat.into_ast());
                Ok(ast::AST::alternation(alt))
            }
        };

        // If we try to pop again, there should be nothing.
        match stack.pop() {
            None => ast,
            Some(GroupState::Alternation(_)) => {
                // This unreachable is unfortunate. This case can't happen
                // because the only way we can be here is if there were two
                // `GroupState::Alternation`s adjacent in the parser's stack,
                // which we guarantee to never happen because we never push a
                // `GroupState::Alternation` if one is already at the top of
                // the stack.
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;

    // Our own assert_eq, which has slightly better formatting (but honestly
    // still kind of crappy).
    macro_rules! assert_eq {
        ($left:expr, $right:expr) => {{
            match (&$left, &$right) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        panic!(
                            "assertion failed: `(left == right)`\n\n\
                             left:  `{:#?}`\nright: `{:#?}`\n\n",
                            left_val, right_val
                        )
                    }
                }
            }
        }};
    }

    fn span(range: Range<usize>) -> ast::Span {
        let start = ast::Position::new(range.start);
        let end = ast::Position::new(range.end);
        ast::Span::new(start, end)
    }

    /// Create a verbatim literal starting at the given position.
    fn lit(c: char, start: usize) -> ast::AST {
        lit_with(c, span(start..start + c.len_utf8()))
    }

    /// Create a verbatim literal with the given span.
    fn lit_with(c: char, span: ast::Span) -> ast::AST {
        ast::AST::literal(ast::Literal {
            span,
            kind: ast::LiteralKind::Verbatim,
            c,
        })
    }

    /// Create a concatenation with the given range.
    fn concat(range: Range<usize>, asts: Vec<ast::AST>) -> ast::AST {
        concat_with(span(range), asts)
    }

    /// Create a concatenation with the given span.
    fn concat_with(span: ast::Span, asts: Vec<ast::AST>) -> ast::AST {
        ast::AST::concat(ast::Concat { span, asts })
    }

    /// Create an alternation with the given span.
    fn alt(range: Range<usize>, asts: Vec<ast::AST>) -> ast::AST {
        ast::AST::alternation(ast::Alternation {
            span: span(range),
            asts,
        })
    }

    #[test]
    fn empty_pattern() {
        assert_eq!(Parser::new().parse(""), Ok(ast::AST::empty(span(0..0))));
    }

    #[test]
    fn single_char_pattern() {
        assert_eq!(Parser::new().parse("a"), Ok(lit('a', 0)));
    }

    #[test]
    fn multiple_chars_pattern_two() {
        assert_eq!(
            Parser::new().parse("ab"),
            Ok(concat(0..2, vec![lit('a', 0), lit('b', 1)]))
        );
    }

    #[test]
    fn multiple_chars_pattern_three() {
        assert_eq!(
            Parser::new().parse("abc"),
            Ok(concat(0..3, vec![lit('a', 0), lit('b', 1), lit('c', 2)]))
        );
    }

    #[test]
    fn alternation_pattern_two_char_options() {
        assert_eq!(
            Parser::new().parse("a|b"),
            Ok(alt(0..3, vec![lit('a', 0), lit('b', 2)]))
        );
    }

    #[test]
    fn alternation_pattern_three_char_options() {
        assert_eq!(
            Parser::new().parse("a|b|c"),
            Ok(alt(0..5, vec![lit('a', 0), lit('b', 2), lit('c', 4)]))
        );
    }

    #[test]
    fn alternation_pattern_multiple_char_options() {
        assert_eq!(
            Parser::new().parse("ab|c|de"),
            Ok(alt(
                0..7,
                vec![
                    concat(0..2, vec![lit('a', 0), lit('b', 1)]),
                    lit('c', 3),
                    concat(5..7, vec![lit('d', 5), lit('e', 6)]),
                ]
            ))
        );
    }
}
