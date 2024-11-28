use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Eq,
    EqEq,
    Comma,
    Colon,
    DoubleColon,
    Semicolon,
    Backslash,
    Arrow,
    Dot,
    Plus,
    Minus,
    Slash,
    Star,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    If,
    Else,
    End,
    Fun,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Eof,
    Identifier,
    Number,
    Symbol(Symbol),
    String,
    Keyword(Keyword),
}

#[derive(Debug, Clone)]
// #[allow(dead_code)]
pub struct Token<'src> {
    pub(crate) kind: TokenKind,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) raw_text: &'src str,
    pub(crate) text: Cow<'src, str>,
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

    #[allow(dead_code)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[allow(dead_code)]
    pub fn end(&self) -> usize {
        self.end
    }

    #[allow(dead_code)]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
    
    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }
    
    #[allow(dead_code)]
    pub fn raw_text(&self) -> &str {
        self.raw_text
    }
}
