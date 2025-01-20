// Experiment with indentation sensitive lexing like in python.

use std::{collections::VecDeque, iter::Peekable};

use lexi_matic::Lexer;

#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "//[^\n]*")]
enum Token0<'a> {
    #[regex("\n? *")]
    Indent(&'a str),
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token<'a> {
    Indent,
    Dedent,
    Ident(&'a str),
}

struct TokenIterator<'a> {
    inner: Peekable<Token0Iterator<'a>>,
    intents: Vec<usize>,
    queue: VecDeque<Token<'a>>,
}

impl<'a> Token<'a> {
    fn lex(input: &'a str) -> TokenIterator<'a> {
        TokenIterator {
            inner: Token0::lex(input).peekable(),
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
                    Token0::Indent(i) => {
                        if matches!(self.inner.peek(), Some(Ok((_, Token0::Indent(_), _)))) {
                            continue;
                        }

                        let spaces = i.strip_prefix('\n').unwrap_or(i).len();
                        let last = self.intents.last().cloned().unwrap_or_default();
                        #[allow(clippy::comparison_chain)]
                        if spaces > last {
                            self.intents.push(spaces);
                            return Some(Ok(Token::Indent));
                        } else if spaces == last {
                            continue;
                        } else {
                            self.intents.pop();
                            loop {
                                let last = self.intents.last().cloned().unwrap_or_default();
                                if spaces > last {
                                    // Misaligned indentation.
                                    self.intents.pop();
                                    self.intents.push(spaces);
                                    self.queue.push_back(Token::Dedent);
                                    self.queue.push_back(Token::Indent);
                                    return Some(Err(lexi_matic::Error(l)));
                                } else if spaces == last {
                                    return Some(Ok(Token::Dedent));
                                } else {
                                    self.intents.pop();
                                    self.queue.push_back(Token::Dedent);
                                }
                            }
                        }
                    }
                    Token0::Ident(i) => return Some(Ok(Token::Ident(i))),
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
  bar
    bar"#,
    );

    for t in it {
        println!("{t:?}");
    }
}
