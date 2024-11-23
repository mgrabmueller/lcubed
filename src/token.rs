use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum Symbol {
    Eq,
    Comma,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum TokenKind {
    Eof,
    Identifier,
    Number,
    Symbol(Symbol),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Token<'src> {
    kind: TokenKind,
    start: usize,
    end: usize,
    raw_text: &'src str,
    text: Cow<'src, str>,
}

impl<'src> Token<'src> {
    #[allow(dead_code)]
    pub fn new(kind: TokenKind) -> Token<'src> {
        Token {
            kind,
            start: 0,
            end: 0,
            raw_text: "",
            text: "".into(),
        }
    }
}