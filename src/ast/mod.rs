// This module was heavily based and copied from https://github.com/rust-lang/regex/blob/master/regex-syntax/src/ast/mod.rs
pub mod parser;

/// An Abstract syntax tree for single regular expression.
#[derive(Debug, PartialEq, Eq)]
pub enum AST {
    /// An empty regex that matches everything.
    Empty(Box<Span>),
    /// A single character literal.
    Literal(Box<Literal>),
    /// An alternation of regular expressions.
    Alternation(Box<Alternation>),
    /// A concatenation of regular expressions.
    Concat(Box<Concat>),
}

impl AST {
    /// Create a "empty" AST item.
    fn empty(e: Span) -> AST {
        AST::Empty(Box::new(e))
    }

    /// Create a "literal" AST item.
    pub fn literal(e: Literal) -> AST {
        AST::Literal(Box::new(e))
    }

    /// Create a "concat" AST item.
    pub fn concat(e: Concat) -> AST {
        AST::Concat(Box::new(e))
    }

    /// Create an "alternation" AST item.
    pub fn alternation(e: Alternation) -> AST {
        AST::Alternation(Box::new(e))
    }
}

/// Represents the position information of a single AST item.
#[derive(Debug, Eq, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    /// Create a new span with the given positions.
    pub fn new(start: Position, end: Position) -> Span {
        Span { start, end }
    }

    /// a new span with the given position as the start and end.
    pub fn splat(pos: Position) -> Span {
        Span::new(pos, pos)
    }
}

/// A single position in a regular expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub offset: usize,
}

impl Position {
    fn new(offset: usize) -> Position {
        Position { offset }
    }
}

/// A single literal expression.
#[derive(Debug, Eq, PartialEq)]
pub struct Literal {
    pub span: Span,
    pub kind: LiteralKind,
    pub c: char,
}

/// The kind of a single literal expression.
#[derive(Debug, Eq, PartialEq)]
pub enum LiteralKind {
    /// The literal is written verbatim.
    Verbatim,
}

/// A concatenation of regular expressions.
#[derive(Debug, Eq, PartialEq)]
pub struct Concat {
    /// The span of the concatenation.
    pub span: Span,
    /// The concatenation regular expressions.
    pub asts: Vec<AST>,
}

impl Concat {
    /// Return a concatenation as an AST.
    fn into_ast(mut self) -> AST {
        match self.asts.len() {
            0 => AST::empty(self.span),
            1 => self.asts.pop().unwrap(),
            _ => AST::concat(self),
        }
    }
}

/// An alternation of regular expressions.
#[derive(Debug, Eq, PartialEq)]
pub struct Alternation {
    /// The span of the alternation.
    pub span: Span,
    /// The alternate regular expressions.
    pub asts: Vec<AST>,
}

impl Alternation {
    /// Return an alternation as an AST.
    fn into_ast(mut self) -> AST {
        match self.asts.len() {
            0 => AST::empty(self.span),
            1 => self.asts.pop().unwrap(),
            _ => AST::alternation(self),
        }
    }
}

/// An error that occurred while parsing a regular expression.
#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of the error.
    kind: ErrorKind,
    /// The original pattern that the parser generated the error from.
    pattern: String,
    /// The span of this error.
    span: Span,
}

/// The type of an error that occurred while building an AST.
#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {}
