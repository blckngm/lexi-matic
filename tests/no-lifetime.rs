use lexi_matic::Lexer;

#[derive(Debug, Lexer, PartialEq, Eq)]
enum Token {
    #[token("import")]
    Import,
    #[token(";")]
    Semi,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    #[regex("//[^\n]*\n")]
    Comment,
    #[regex(r"[ \t\r\n\f]+")]
    Space,
}

#[test]
fn test_tokens() {
    let input = Token::lex("import // ...\nimport1;");
    let expected = [
        (0, Token::Import, 6),
        (6, Token::Space, 7),
        (7, Token::Comment, 14),
        (14, Token::Ident, 21),
        (21, Token::Semi, 22),
    ];
    let mut seen = 0;
    for (i, t) in input.enumerate() {
        let r = t.unwrap();
        assert_eq!(r, expected[i]);
        seen += 1;
    }
    assert_eq!(seen, expected.len());
}
