# Lexi-Matic

A proc-macro for lexers similar to logos. Uses regex-automata DFA under the
hood.

```rust
# use lexi_matic::Lexer;
#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "//[^\n]*\n", skip = r"[ \t\r\n\f]+")]
enum Token<'a> {
    #[token("import")]
    Import,
    #[token(";")]
    Semi,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}

// An iterator of Result<(usize, Token, usize), lexi_matic::Error>.
let tokens = Token::lex("import foo_bar;import import1;// ...\nimport buz;");
for t in tokens {
    let (start, t, end) = t.unwrap();
    println!("{start}..{end} {t:?}");
}
```

## Token Disambiguation

There are only two simple rules:

* Longer matches always win.
* If multiple patterns are matched for the longest match, the *first* pattern wins.

So if you have keywords and identifiers, specify the keywords *first*:

```rust
# use lexi_matic::Lexer;
#[derive(Lexer)]
enum Token<'a> {
    #[token("import")]
    Import,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}
```

So `import` would be `Import` but `import1` would be `Ident`.
