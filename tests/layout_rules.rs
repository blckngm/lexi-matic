// Experiment with layout rules like in Haskell.
//
// But not exactly like in Haskell, because we will close layouts when seeing `in` or `)` instead of using `parse-error`.

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
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
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
    LParen,
    RParen,
}

impl<'a> Token<'a> {
    fn lex(input: &'a str) -> TokenIterator<'a> {
        TokenIterator {
            inner: RawToken::lex(input).peekable(),
            layouts: Default::default(),
            pending_layout: None,
            column: 0,
            queue: Default::default(),
        }
    }
}

enum Layout {
    Let(usize),
    Other(usize),
    Paren,
}

impl Layout {
    fn column(&self) -> Option<usize> {
        match self {
            Self::Let(c) => Some(*c),
            Self::Other(c) => Some(*c),
            Self::Paren => None,
        }
    }

    fn with_column(self, c: usize) -> Self {
        match self {
            Self::Let(_) => Self::Let(c),
            Self::Other(_) => Self::Other(c),
            Self::Paren => Self::Paren,
        }
    }

    fn is_paren(&self) -> bool {
        matches!(self, Self::Paren)
    }

    fn is_let(&self) -> bool {
        matches!(self, Self::Let(_))
    }
}

struct TokenIterator<'a> {
    inner: Peekable<RawTokenIterator<'a>>,
    layouts: Vec<Layout>,
    pending_layout: Option<Layout>,
    column: usize,
    queue: VecDeque<Token<'a>>,
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
                Some(Ok((l, t, r))) => {
                    if !matches!(t, RawToken::Whitespace(_) | RawToken::Indent(_)) {
                        if let Some(l) = self.pending_layout.take() {
                            self.layouts.push(l.with_column(self.column));
                        }
                    }
                    self.column += r - l;
                    match t {
                        RawToken::Let => {
                            self.queue.push_back(Token::VLBrace);
                            self.pending_layout = Some(Layout::Let(0));
                        }
                        RawToken::By => {
                            self.queue.push_back(Token::VLBrace);
                            self.pending_layout = Some(Layout::Other(0));
                        }
                        _ => {}
                    }
                    match t {
                        RawToken::Whitespace(_) => continue,
                        RawToken::Indent(indent) => {
                            let col = indent.len() - 1;
                            self.column = col;

                            // Ignore this indentation if the line is effectively empty or if the next token is `in`.
                            // For `in` we will handle the closing in the next iteration.
                            if let Some(Ok((_, RawToken::Indent(_) | RawToken::In, _))) =
                                self.inner.peek()
                            {
                                continue;
                            }

                            // Ignore this indentation if the reference column of the new layout block hasn't been determined yet.
                            if self.pending_layout.is_some() {
                                continue;
                            }

                            if let Some(last) = self.layouts.last() {
                                if Some(col) == last.column() {
                                    return Some(Ok(Token::VSemicolon));
                                }
                            }

                            if Some(col) < self.layouts.last().and_then(|l| l.column()) {
                                self.layouts.pop();

                                while Some(col) < self.layouts.last().and_then(|l| l.column()) {
                                    self.layouts.pop();
                                    self.queue.push_back(Token::VRBrace);
                                }

                                return Some(Ok(Token::VRBrace));
                            } else {
                                continue;
                            }
                        }
                        RawToken::ColonEqual => return Some(Ok(Token::ColonEqual)),
                        RawToken::Let => return Some(Ok(Token::Let)),
                        RawToken::By => return Some(Ok(Token::By)),
                        RawToken::LParen => {
                            self.layouts.push(Layout::Paren);
                            return Some(Ok(Token::LParen));
                        }
                        RawToken::RParen => {
                            // Close layouts until we find the matching paren.
                            self.close_layouts_until(Layout::is_paren);
                            self.queue.push_back(Token::RParen);
                            return Some(Ok(self.queue.pop_front().unwrap()));
                        }
                        RawToken::In => {
                            // Close layouts until we find the closest `let`.
                            if self.close_layouts_until(Layout::is_let) {
                                self.queue.push_back(Token::VRBrace);
                            }
                            self.queue.push_back(Token::In);
                            return Some(Ok(self.queue.pop_front().unwrap()));
                        }
                        RawToken::Identifier(i) => return Some(Ok(Token::Identifier(i))),
                    }
                }
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

impl TokenIterator<'_> {
    fn close_layouts_until(&mut self, p: impl Fn(&Layout) -> bool) -> bool {
        loop {
            match self.layouts.pop() {
                Some(l) if p(&l) => return true,
                // Unmatched paren. Do nothing.
                Some(l) if l.is_paren() => {}
                Some(_) => self.queue.push_back(Token::VRBrace),
                None => return false,
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

#[test]
fn test_let_by_layout() {
    use Token::*;

    let it = Token::lex(
        r#"let x := g by
  y
  z
in f x"#,
    );

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Identifier("g"),
            By,
            VLBrace,
            Identifier("y"),
            VSemicolon,
            Identifier("z"),
            VRBrace,
            VRBrace,
            In,
            Identifier("f"),
            Identifier("x"),
        ],
    );
}

#[test]
fn test_let_by_single_line() {
    use Token::*;

    // In should close both by and let.
    let it = Token::lex("let x := p by a b in x");

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Identifier("p"),
            By,
            VLBrace,
            Identifier("a"),
            Identifier("b"),
            VRBrace,
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test_parens() {
    use Token::*;

    let it = Token::lex(
        r#"
let x := f (g
           y) (let z := w
               in z)
    y := (a b
-- Layout rule is disabled in parens.
c)
in x"#,
    );

    assert_tokens(
        it,
        [
            Let,
            VLBrace,
            Identifier("x"),
            ColonEqual,
            Identifier("f"),
            LParen,
            Identifier("g"),
            Identifier("y"),
            RParen,
            LParen,
            Let,
            VLBrace,
            Identifier("z"),
            ColonEqual,
            Identifier("w"),
            VRBrace,
            In,
            Identifier("z"),
            RParen,
            VSemicolon,
            Identifier("y"),
            ColonEqual,
            LParen,
            Identifier("a"),
            Identifier("b"),
            Identifier("c"),
            RParen,
            VRBrace,
            In,
            Identifier("x"),
        ],
    );
}

#[test]
fn test_by_in_parens() {
    use Token::*;

    let it = Token::lex(
        r#"
-- The second right paren should close the by block.
f (a by y (foo bar)) b
"#,
    );

    assert_tokens(
        it,
        [
            Identifier("f"),
            LParen,
            Identifier("a"),
            By,
            VLBrace,
            Identifier("y"),
            LParen,
            Identifier("foo"),
            Identifier("bar"),
            RParen,
            VRBrace,
            RParen,
            Identifier("b"),
        ],
    );
}

fn assert_tokens<'a>(it: TokenIterator<'a>, expected: impl IntoIterator<Item = Token<'a>>) {
    itertools::assert_equal(
        it.map(|r| r.map_err(|e| format!("{e:?}"))),
        expected.into_iter().map(Ok),
    );
}
