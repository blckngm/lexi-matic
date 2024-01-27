#![feature(test)]

extern crate test;

use lexi_matic::Lexer;
use logos::Logos;

#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "//[^\n]*", skip = r"[ \t\r\n\f]+")]
enum Token<'a> {
    #[token("import")]
    Import,
    #[token(";")]
    Semi,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}

#[derive(Debug, Logos, PartialEq, Eq)]
#[logos(skip "//[^\n]*", skip r"[ \t\r\n\f]+")]
enum TokenLogos<'a> {
    #[token("import")]
    Import,
    #[token(";")]
    Semi,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}

const INPUT: &str = r####"import import1;
// ..................................
import something_else;
import something_else1;
"####;

#[bench]
fn bench_lex(b: &mut test::Bencher) {
    let mut tokens = Token::lex(INPUT);

    b.bytes = INPUT.len() as u64;
    b.iter(|| {
        tokens.consumed = 0;
        tokens.by_ref().count()
    });
}

#[bench]
fn bench_logos(b: &mut test::Bencher) {
    b.bytes = INPUT.len() as u64;
    b.iter(|| TokenLogos::lexer(INPUT).by_ref().count());
}
