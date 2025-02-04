// Experiment with layout rules like in Haskell.
//
// But not exactly like in Haskell, because we will stop layouts when seeing `in` instead of using `parse-error`.

use std::{collections::VecDeque, iter::Peekable};

use lexi_matic::{Error as LexerError, Lexer};

#[derive(Debug, Lexer, PartialEq, Eq)]
#[lexer(skip = "--[^\n]*")]
enum RawToken<'a> {
    #[regex("\n *")]
    Indent(&'a str),
    #[regex(" +")]
    Whitespace(&'a str),
    #[token(":=")]
    ColonEqual,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("by")]
    By,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token<'a> {
    Identifier(&'a str),
    VLBrace,
    VRBrace,
    VSemicolon,
    ColonEqual,
    Let,
    In,
    By,
}

impl<'a> Token<'a> {
    fn lex(input: &'a str) -> TokenIterator<'a> {
        TokenIterator {
            inner: RawToken::lex(input).peekable(),
            layouts: Default::default(),
            layout_just_started: false,
            column: 0,
            queue: Default::default(),
        }
    }
}

struct TokenIterator<'a> {
    inner: Peekable<RawTokenIterator<'a>>,
    layouts: Vec<usize>,
    layout_just_started: bool,
    column: usize,
    queue: VecDeque<Token<'a>>,
}

impl TokenIterator<'_> {
    fn maybe_push_layout(&mut self) {
        if self.layout_just_started {
            self.layout_just_started = false;
            self.layouts.push(self.column);
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<Token<'a>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.queue.pop_front() {
            return Some(Ok(t));
        }

        loop {
            match self.inner.next() {
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok((_, t, _))) => match t {
                    RawToken::Whitespace(w) => {
                        self.column += w.len();
                        continue;
                    }
                    RawToken::Indent(indent) => {
                        let col = indent.len() - 1;
                        self.column = col;

                        // Ignore this indentation if it's effectively empty or if the next token is `in`.
                        // For `in` we will pop and insert VRBrace regardless of the indentation.
                        if let Some(Ok((_, RawToken::Indent(_) | RawToken::In, _))) =
                            self.inner.peek()
                        {
                            continue;
                        }

                        // Ignore this indentation if the reference column of the new layout block hasn't been determined yet.
                        if self.layout_just_started {
                            continue;
                        }

                        if let Some(last) = self.layouts.last() {
                            if col == *last {
                                return Some(Ok(Token::VSemicolon));
                            }
                        }

                        if Some(&col) < self.layouts.last() {
                            self.layouts.pop();

                            while Some(&col) < self.layouts.last() {
                                self.layouts.pop();
                                self.queue.push_back(Token::VRBrace);
                            }

                            return Some(Ok(Token::VRBrace));
                        } else {
                            continue;
                        }
                    }
                    RawToken::ColonEqual => {
                        self.maybe_push_layout();
                        self.column += 2;

                        return Some(Ok(Token::ColonEqual));
                    }
                    RawToken::Let => {
                        self.maybe_push_layout();
                        self.column += 3;

                        self.queue.push_back(Token::VLBrace);
                        self.layout_just_started = true;
                        return Some(Ok(Token::Let));
                    }
                    RawToken::By => {
                        self.maybe_push_layout();
                        self.column += 2;

                        self.queue.push_back(Token::VLBrace);
                        self.layout_just_started = true;
                        return Some(Ok(Token::By));
                    }
                    RawToken::In => {
                        self.maybe_push_layout();
                        self.column += 2;

                        // Pop a layout and insert an VRBrace if there are any layouts.
                        if !self.layouts.is_empty() {
                            self.layouts.pop();
                            self.queue.push_back(Token::In);
                            return Some(Ok(Token::VRBrace));
                        }
                        return Some(Ok(Token::In));
                    }
                    RawToken::Identifier(i) => {
                        self.maybe_push_layout();
                        self.column += i.len();

                        return Some(Ok(Token::Identifier(i)));
                    }
                },
                None => {
                    if !self.layouts.is_empty() {
                        for _ in self.layouts.drain(..).skip(1) {
                            self.queue.push_back(Token::VRBrace);
                        }
                        return Some(Ok(Token::VRBrace));
                    }
                    return None;
                }
            }
        }
    }
}

#[test]
fn test() {
    use Token::*;

    let it = Token::lex(
        r#"
let
  x := x
    y y
 -- some misaligned comment
  z := z
in x"#,
    );

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Identifier("x"),
            Identifier("y"),
            Identifier("y"),
            VSemicolon,
            Identifier("z"),
            ColonEqual,
            Identifier("z"),
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test1() {
    use Token::*;

    // This is different from test because x := x is on the same line as the `let`.
    let it = Token::lex(
        r#"
let   x := x
          y y
   -- some misaligned comment
      z := z
in x"#,
    );

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Identifier("x"),
            Identifier("y"),
            Identifier("y"),
            VSemicolon,
            Identifier("z"),
            ColonEqual,
            Identifier("z"),
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test_let_in_same_line() {
    use Token::*;

    let it = Token::lex(r#"let x := let y := x in y in x"#);

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Let,
            VLBrace,
            Identifier("y"),
            ColonEqual,
            Identifier("x"),
            VRBrace,
            In,
            Identifier("y"),
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test_nested_let() {
    use Token::*;

    let it = Token::lex(
        r#"
let x := let y := y
         in y
in x"#,
    );

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Let,
            VLBrace,
            Identifier("y"),
            ColonEqual,
            Identifier("y"),
            VRBrace,
            In,
            Identifier("y"),
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test_by_layout() {
    use Token::*;
    let it = Token::lex(
        r#"
by foo by
  x
    y
  z
-- bar should pop two layout blocks
bar by foo by
            x
                y
            z
       -- bar should pop one layout block
       bar
"#,
    );

    assert_tokens(
        it,
        [
            By,
            VLBrace,
            Identifier("foo"),
            By,
            VLBrace,
            Identifier("x"),
            Identifier("y"),
            VSemicolon,
            Identifier("z"),
            VRBrace,
            VRBrace,
            Identifier("bar"),
            By,
            VLBrace,
            Identifier("foo"),
            By,
            VLBrace,
            Identifier("x"),
            Identifier("y"),
            VSemicolon,
            Identifier("z"),
            VRBrace,
            Identifier("bar"),
            VRBrace,
        ],
    );
}

fn assert_tokens<'a>(it: TokenIterator<'a>, expected: impl IntoIterator<Item = Token<'a>>) {
    itertools::assert_equal(
        it.map(|r| r.map_err(|e| format!("{e:?}"))),
        expected.into_iter().map(Ok),
    );
}
