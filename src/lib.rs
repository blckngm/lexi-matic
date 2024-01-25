#![doc = include_str!("../README.md")]
use std::fmt;

use regex_automata::{dfa::Automaton, PatternID};

pub use lexi_matic_derive::Lexer;
#[doc(hidden)]
pub use regex_automata::dfa::dense::DFA;

#[derive(Debug)]
pub struct Error(pub usize);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexical error at {}", self.0)
    }
}

impl std::error::Error for Error {}

pub trait Lexer<'a>: Sized {
    type Iterator: IntoIterator<Item = Result<(usize, Self, usize), Error>>;
    fn lex(input: &'a str) -> Self::Iterator;
}

#[doc(hidden)]
pub fn dfa_search_next(dfa: &DFA<&[u32]>, input: &str) -> Option<(PatternID, usize)> {
    let m = dfa
        .try_search_fwd(&regex_automata::Input::new(input).anchored(regex_automata::Anchored::Yes))
        .ok()??;
    Some((m.pattern(), m.offset()))
}
