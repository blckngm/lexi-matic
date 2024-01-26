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

## Custom Lexing

Sometimes the lexing grammar isn't regular or even
[context-free](https://github.com/rust-lang/rust/blob/HEAD@%7B2019-05-26T21:45:17Z%7D/src/grammar/raw-string-literal-ambiguity.md). You can use a callback for these:

```rust
# use lexi_matic::Lexer;

#[derive(Debug, Lexer)]
enum Token<'a> {
    #[token(";")]
    Semi,
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
```