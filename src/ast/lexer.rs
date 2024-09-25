use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EOF,
    Char, // chars
    // CharacterClass, // [...]
    // ClassRange,     // a-z or 0-9
    Unknown,
    Whitespace,
    Bar,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Char => write!(f, "char"),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    // type:
    //     | 'whitespace'
    //     | 'comment-inline'
    //     | 'comment-block'
    //     | 'string'
    //     | 'semicolon'
    //     | 'keyword'
    //     | 'parameter'
    //     | 'table'
    //     | 'unknown';
    pub kind: TokenKind,
    pub span: TextSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: TextSpan) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub literal: String,
}

impl TextSpan {
    pub fn new(start: usize, end: usize, literal: String) -> Self {
        Self {
            start,
            end,
            literal,
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            current_pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.current_pos == self.input.len() {
            let eof_char: char = '\0';
            self.current_pos += 1;
            return Some(Token::new(
                TokenKind::EOF,
                TextSpan::new(0, 0, eof_char.to_string()),
            ));
        }

        let c = self.current_char();

        c.map(|c| {
            println!("c {}", c);
            let start = self.current_pos;
            let kind: TokenKind;

            if Self::is_whitespace(&c) {
                self.consume();
                kind = TokenKind::Whitespace;
            } else if c.is_alphabetic() {
                kind = self.consume_char();
            } else {
                kind = match c {
                    '|' => TokenKind::Bar,
                    _ => TokenKind::Unknown,
                };
                self.consume();
            }

            let end = self.current_pos;
            let literal = self.input[start..end].to_string();
            let span = TextSpan::new(start, end, literal);

            Token::new(kind, span)
        })
    }

    pub fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.current_pos)
    }

    fn is_whitespace(c: &char) -> bool {
        c.is_whitespace()
    }

    fn consume(&mut self) -> Option<char> {
        if self.current_pos >= self.input.len() {
            return None;
        }

        let c = self.current_char();
        self.current_pos += 1;

        c
    }

    fn consume_char(&mut self) -> TokenKind {
        self.consume().unwrap();
        TokenKind::Char
    }
}

pub fn tokens(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        dbg!(&token);
        tokens.push(token);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        let input = "a|b";
        let tokens = tokens(&input);
        println!("Tokens: {:#?}", tokens);
        // assert_eq!(tokens, vec![]);
    }
}
