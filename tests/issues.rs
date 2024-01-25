// Test that we don't have these logos issues.

use lexi_matic::Lexer;

#[test]
fn test_279() {
    #[derive(Lexer, Debug)]
    enum Token {
        #[token = r"\"]
        Backslash,
        #[token = r"\\"]
        DoubleBackslash,
        #[token = r"\begin"]
        EnvironmentBegin,
        #[token = r"\end"]
        EnvironmentEnd,
        #[token = r"\begin{document}"]
        DocumentBegin,
        #[regex = r"\\[a-zA-Z]+"]
        MacroName,
    }

    let mut tokens = Token::lex(r"\begin{equation}");
    let t = tokens.next().unwrap().unwrap();
    assert!(matches!(t, (0, Token::EnvironmentBegin, 6)));
}

#[test]
fn test_315() {
    #[derive(Lexer, Clone, Copy, Debug, PartialEq)]
    enum Token {
        #[token = "a"]
        A,
        #[token = "b"]
        B,
        #[regex = r"[ab]*c"]
        Abc,
    }

    assert!(Token::lex("aba")
        .map(|r| r.unwrap().1)
        .eq([Token::A, Token::B, Token::A,]));
}

#[test]
fn test_349() {
    #[derive(Lexer, Debug, PartialEq, Eq)]
    enum Foo {
        #[token = "FOOB"]
        Foo,
    }

    let mut lex = Foo::lex("ZAP");
    assert!(lex.next().unwrap().is_err());
}
