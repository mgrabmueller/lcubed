use crate::{scanner::{ScanError, Scanner}, token::{Symbol, TokenKind}};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    ScanError(ScanError),
    Unexpected{expected: TokenKind, found: TokenKind},
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ScanError(e) => {
                e.fmt(f)
            }
            ParseError::Unexpected { expected, found } => {
                write!(f, "expected {expected:?}, found {found:?} instead")
            }
        }
    }
}

impl From<ScanError> for ParseError {
    fn from(err: ScanError) -> Self {
        ParseError::ScanError(err)
    }
}

pub struct Parser<'src> {
    scanner: Scanner<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(input: &'src str) -> Result<Parser<'src>, ParseError> {
        let scanner = Scanner::new(input)?;
        Ok(Parser { scanner })
    }

    fn accept(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.scanner.token().kind() == kind {
            let _ = self.scanner.scan()?;
            Ok(())
        } else {
            Err(ParseError::Unexpected{expected: kind, found: self.scanner.token().kind()})
        }
    }
    pub fn parse_program(&mut self) -> Result<(), ParseError> {
        self.accept(TokenKind::Identifier)?;
        self.accept(TokenKind::Symbol(Symbol::DoubleColon))?;
        self.accept(TokenKind::Identifier)?;
        self.accept(TokenKind::Symbol(Symbol::Semicolon))?;
        self.accept(TokenKind::Identifier)?;
        self.accept(TokenKind::Symbol(Symbol::Eq))?;
        self.accept(TokenKind::Number)?;
        self.accept(TokenKind::Symbol(Symbol::Semicolon))?;
        self.accept(TokenKind::Eof)?;
        Ok(())
    }
}
