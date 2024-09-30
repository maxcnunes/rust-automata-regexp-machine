use crate::{
    ast::{parser::Parser, AST},
    automata::{dfa::DFA, nfa::NFA},
    error::Error,
};

pub struct Regex {
    dfa: DFA,
}

impl Regex {
    pub fn new(input: &str) -> Result<Regex, Error> {
        let ast = Parser::new()
            .parse(input)
            .map_err(Error::from_ast_parse_error)?;
        let nfa = ast_to_nfa(&ast);
        let dfa = nfa_to_dfa(&nfa);
        // dfa.minimize();
        Ok(Regex { dfa })
    }

    pub fn test(&self, text: &str) -> bool {
        self.dfa.test(text)
    }
}

/// Translates AST to NFA.
fn ast_to_nfa(ast: &AST) -> NFA {
    // This uses recursion to walk all the nested nodes, which isn't ideal if we were planing
    // to support large trees, and the original Rust implementation uses a visitor implementation instead.
    // But to keep this simple we use recursion for this implementation.
    match ast {
        AST::Literal(lit) => NFA::char(lit.c),
        AST::Concat(con) => NFA::concat(con.asts.iter().map(|a| ast_to_nfa(&a)).collect()),
        AST::Alternation(alt) => NFA::or(alt.asts.iter().map(|a| ast_to_nfa(&a)).collect()),
        AST::Empty(_) => NFA::empty(),
    }
}

/// Translates NFA to DFA.
fn nfa_to_dfa(nfa: &NFA) -> DFA {
    DFA::from(nfa)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::AST, error::Error};

    fn parse_ast(input: &str) -> Result<AST, Error> {
        Parser::new()
            .parse(input)
            .map_err(Error::from_ast_parse_error)
    }

    #[test]
    fn ast_to_nfa_empty() {
        let ast = parse_ast("").unwrap();
        dbg!(&ast);

        let nfa = ast_to_nfa(&ast);
        dbg!(&nfa);

        assert_eq!(nfa, NFA::empty());
    }

    #[test]
    fn ast_to_nfa_single_char() {
        let ast = parse_ast("a").unwrap();
        dbg!(&ast);

        let nfa = ast_to_nfa(&ast);
        dbg!(&nfa);

        assert_eq!(nfa, NFA::char('a'));
    }

    #[test]
    fn ast_to_nfa_multiple_chars() {
        let ast = parse_ast("ab").unwrap();
        dbg!(&ast);

        let nfa = ast_to_nfa(&ast);
        dbg!(&nfa);

        assert_eq!(nfa, NFA::concat(vec![NFA::char('a'), NFA::char('b')]));
    }

    #[test]
    fn ast_to_nfa_multiple_chars_with_alternation() {
        let ast = parse_ast("ab|c").unwrap();
        dbg!(&ast);

        let nfa = ast_to_nfa(&ast);
        dbg!(&nfa);

        assert_eq!(
            nfa,
            NFA::or(vec![
                NFA::concat(vec![NFA::char('a'), NFA::char('b')]),
                NFA::char('c'),
            ],)
        );
    }
    //
    // #[test]
    // fn test_empty() {
    //     assert_eq!(Regex::new("").unwrap().test(""), true);
    // }

    #[test]
    fn test_single_char() {
        assert_eq!(Regex::new("a").unwrap().test("a"), true);
    }

    #[test]
    fn test_multiple_chars() {}

    #[test]
    fn test_multiple_chars_with_alternation() {}
}
