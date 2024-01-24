use std::fmt;

pub use lexi_matic_derive::Lexer;
use regex_automata::{
    dfa::{dense::DFA, Automaton},
    PatternID,
};

#[derive(Debug)]
pub struct Error(pub usize);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexical error at {}", self.0)
    }
}

impl std::error::Error for Error {}

pub trait Lexer<'a> {
    type Iterator;
    fn lex(input: &'a str) -> Self::Iterator;
}

pub fn dfa_search_next(dfa: &DFA<&[u32]>, input: &str) -> Option<(PatternID, usize)> {
    let m = dfa
        .try_search_fwd(&regex_automata::Input::new(input).anchored(regex_automata::Anchored::Yes))
        .ok()??;
    Some((m.pattern(), m.offset()))
}
