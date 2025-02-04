// Experiment with indentation sensitive lexing like in python.

use std::{cmp::Ordering, collections::VecDeque, fmt, iter::Peekable};

use lexi_matic::Lexer;

#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "//[^\n]*")]
enum RawToken<'a> {
    #[regex("\n *")]
    Indent(&'a str),
    #[regex(" +")]
    Whitespace(&'a str),
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token<'a> {
    Indent,
    Dedent,
    Identifier(&'a str),
    LBracket,
    RBracket,
    Comma,
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    MisalignedIndentation(usize),
    LexicalError(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MisalignedIndentation(i) => write!(f, "Misaligned indentation at {}", i),
            Self::LexicalError(i) => write!(f, "Lexical error at {}", i),
        }
    }
}

impl std::error::Error for Error {}

struct TokenIterator<'a> {
    inner: Peekable<RawTokenIterator<'a>>,
    intents: Vec<usize>,
    brackets: usize,
    queue: VecDeque<Token<'a>>,
}

impl<'a> Token<'a> {
    fn lex(input: &'a str) -> TokenIterator<'a> {
        TokenIterator {
            inner: RawToken::lex(input).peekable(),
            brackets: 0,
            intents: Default::default(),
            queue: Default::default(),
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.queue.pop_front() {
            return Some(Ok(t));
        }

        loop {
            match self.inner.next() {
                Some(Err(e)) => return Some(Err(Error::LexicalError(e.0))),
                Some(Ok((l, t, _))) => match t {
                    RawToken::Whitespace(w) => {
                        // Whitespace at the start of input is indentation.
                        if l == 0 {
                            self.intents.push(w.len());
                            return Some(Ok(Token::Indent));
                        }
                    }
                    RawToken::Indent(indent) => {
                        if self.brackets > 0 {
                            continue;
                        }
                        if matches!(
                            self.inner.peek(),
                            Some(Ok((_, RawToken::Indent(_) | RawToken::Whitespace(_), _)))
                        ) {
                            continue;
                        }

                        let level = indent.len() - 1;
                        let last = self.intents.last().cloned().unwrap_or_default();
                        match level.cmp(&last) {
                            Ordering::Greater => {
                                self.intents.push(level);
                                return Some(Ok(Token::Indent));
                            }
                            Ordering::Equal => continue,
                            Ordering::Less => {
                                // We pop without enqueueing a dedent token here because we'll return
                                // one directly when we find the matching level in the loop below
                                self.intents.pop();
                                loop {
                                    let last = self.intents.last().cloned().unwrap_or_default();
                                    match level.cmp(&last) {
                                        Ordering::Greater => {
                                            // Misaligned indentation.
                                            self.intents.pop();
                                            self.intents.push(level);
                                            // When we detect misaligned indentation, we emit a DEDENT + INDENT pair
                                            // to maintain proper block structure while still indicating an error occurred
                                            self.queue.push_back(Token::Dedent);
                                            self.queue.push_back(Token::Indent);
                                            return Some(Err(Error::MisalignedIndentation(l)));
                                        }
                                        Ordering::Equal => {
                                            return Some(Ok(Token::Dedent));
                                        }
                                        Ordering::Less => {
                                            self.intents.pop();
                                            self.queue.push_back(Token::Dedent);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    RawToken::LBracket => {
                        self.brackets += 1;
                        return Some(Ok(Token::LBracket));
                    }
                    RawToken::RBracket => {
                        self.brackets = self.brackets.saturating_sub(1);
                        return Some(Ok(Token::RBracket));
                    }
                    RawToken::Comma => return Some(Ok(Token::Comma)),
                    RawToken::Identifier(i) => return Some(Ok(Token::Identifier(i))),
                },
                None => {
                    if !self.intents.is_empty() {
                        for _ in self.intents.drain(..).skip(1) {
                            self.queue.push_back(Token::Dedent);
                        }
                        return Some(Ok(Token::Dedent));
                    }
                    return None;
                }
            }
        }
    }
}

#[test]
fn test() {
    let it = Token::lex(
        r#"
foo
    bar
        baz
  bar  // xxx.
 //
  bar [
    x,
    y,
    z,
  ]
  bar
    baz"#,
    );

    let expected = [
        Ok(Token::Identifier("foo")),
        Ok(Token::Indent),
        Ok(Token::Identifier("bar")),
        Ok(Token::Indent),
        Ok(Token::Identifier("baz")),
        Err(Error::MisalignedIndentation(24)),
        Ok(Token::Dedent),
        Ok(Token::Dedent),
        Ok(Token::Indent),
        Ok(Token::Identifier("bar")),
        Ok(Token::Identifier("bar")),
        Ok(Token::LBracket),
        Ok(Token::Identifier("x")),
        Ok(Token::Comma),
        Ok(Token::Identifier("y")),
        Ok(Token::Comma),
        Ok(Token::Identifier("z")),
        Ok(Token::Comma),
        Ok(Token::RBracket),
        Ok(Token::Identifier("bar")),
        Ok(Token::Indent),
        Ok(Token::Identifier("baz")),
        Ok(Token::Dedent),
        Ok(Token::Dedent),
    ];

    for (i, (actual, expected)) in it.zip(expected).enumerate() {
        assert_eq!(actual, expected, "Mismatch at index {i}");
    }
}
