#![doc = include_str!("../README.md")]
use std::fmt;

pub use lexi_matic_derive::Lexer;
#[doc(hidden)]
pub use regex_automata::dfa::dense::DFA;
use regex_automata::{dfa::Automaton, util::start::Config, PatternID};

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
    let start = dfa
        .start_state(&Config::new().anchored(regex_automata::Anchored::Yes))
        .unwrap();
    let mut state = start;
    let mut matched = (start, 0);
    'search: {
        for (i, b) in input.as_bytes().iter().copied().enumerate() {
            state = dfa.next_state(state, b);
            if dfa.is_match_state(state) {
                matched = (state, i);
            } else if dfa.is_dead_state(state) {
                break 'search;
            }
        }
        state = dfa.next_eoi_state(state);
        if dfa.is_match_state(state) {
            matched = (state, input.len());
        }
    }
    if matched.1 != 0 {
        Some((dfa.match_pattern(matched.0, 0), matched.1))
    } else {
        None
    }
}
