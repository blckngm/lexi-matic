// Experiment with indentation sensitive lexing like in python.

use std::{collections::VecDeque, iter::Peekable};

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
    type Item = Result<Token<'a>, lexi_matic::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.queue.pop_front() {
            return Some(Ok(t));
        }

        loop {
            match self.inner.next() {
                Some(Err(e)) => return Some(Err(e)),
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
                        #[allow(clippy::comparison_chain)]
                        if level > last {
                            self.intents.push(level);
                            return Some(Ok(Token::Indent));
                        } else if level == last {
                            continue;
                        } else {
                            self.intents.pop();
                            loop {
                                let last = self.intents.last().cloned().unwrap_or_default();
                                if level > last {
                                    // Misaligned indentation.
                                    self.intents.pop();
                                    self.intents.push(level);
                                    self.queue.push_back(Token::Dedent);
                                    self.queue.push_back(Token::Indent);
                                    return Some(Err(lexi_matic::Error(l)));
                                } else if level == last {
                                    return Some(Ok(Token::Dedent));
                                } else {
                                    self.intents.pop();
                                    self.queue.push_back(Token::Dedent);
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

    for t in it {
        println!("{t:?}");
    }
}
