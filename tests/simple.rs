use lexi_matic::Lexer;

#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "//[^\n]*", skip = r"[ \t\r\n\f]+")]
enum Token<'a> {
    #[token("import")]
    Import,
    #[token(";")]
    Semi,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
    #[regex(r##"r#*""##)]
    #[lexer(more = end_raw_str)]
    RawStr(&'a str),
}

// A `more` function should return how many more bytes to include in this token.
// We are trying to finish a raw string literal, so we search for the matching
// `"###`.
//
// If a `more` function returns `None`, it is considered a lexical error.
fn end_raw_str(matched: &str, remaining: &str) -> Option<usize> {
    let start: String = matched[1..].chars().rev().collect();
    remaining.find(&start).map(|l| l + start.len())
}

#[test]
fn test_tokens() {
    let input = Token::lex(
        r####"import // ...
import1; r#"abc"#"####,
    );
    let expected = [
        (0, Token::Import, 6),
        (14, Token::Ident("import1"), 21),
        (21, Token::Semi, 22),
        (23, Token::RawStr(r###"r#"abc"#"###), 31),
    ];
    let mut seen = 0;
    for (i, t) in input.enumerate() {
        let r = t.unwrap();
        assert_eq!(r, expected[i]);
        seen += 1;
    }
    assert_eq!(seen, expected.len());
}
