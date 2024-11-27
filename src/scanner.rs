use std::str::CharIndices;

use crate::token::{Symbol, Token, TokenKind};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ScanError {
    UnexpectedEndOfInput { offset: usize },
    UnexpectedCharacter { offset: usize, unexpected: char },
    UnexpectedCharacterInEscapeSequence { offset: usize, unexpected: char },
    UnexpectedEndOfInputInString { offset: usize, string_start: usize },
    UnexpectedEndOfInputInEscapeSequence { offset: usize },
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedEndOfInput { offset } => {
                write!(f, "unexpected end of input at offset {offset}")
            }
            ScanError::UnexpectedCharacter { offset, unexpected } => {
                write!(f, "unexpected character {unexpected:?} at offset {offset}")
            }
            ScanError::UnexpectedEndOfInputInString {
                offset,
                string_start,
            } => {
                write!(f, "unexpected end of input at offset {offset} in string starting at {string_start}")
            }
            ScanError::UnexpectedCharacterInEscapeSequence { offset, unexpected } => {
                write!(f, "unexpected character {unexpected:?} in escape sequence at offset {offset}")
            }
            ScanError::UnexpectedEndOfInputInEscapeSequence {
                offset
            } => {
                write!(f, "unexpected end of input at offset {offset} in escape sequence")
            }
        }
    }
}

pub struct Scanner<'src> {
    input: &'src str,
    chars: CharIndices<'src>,
    last_char: Option<char>,
    current_char: Option<char>,
    position: usize,
    token: Token<'src>,
}

impl<'src> Scanner<'src> {
    pub fn new(input: &'src str) -> Result<Scanner<'src>, ScanError> {
        let mut scanner = Scanner {
            input,
            chars: input.char_indices(),
            last_char: None,
            current_char: None,
            position: 0,
            token: Token::new(TokenKind::Eof),
        };
        scanner.scan_char()?;
        scanner.scan()?;
        Ok(scanner)
    }

    fn scan_char(&mut self) -> Result<(), ScanError> {
        if let Some((ofs, ch)) = self.chars.next() {
            self.last_char = self.current_char;
            self.current_char = Some(ch);
            self.position = ofs;
        } else {
            self.position += self.last_char.map_or(1, |c| c.len_utf8());
            self.current_char = None;
        }
        Ok(())
    }

    fn skip_whitespace(&mut self) -> Result<(), ScanError> {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() {
                break;
            }
            self.scan_char()?;
        }
        Ok(())
    }

    fn finish_token(&mut self, kind: TokenKind) -> Result<(), ScanError> {
        self.token.kind = kind;
        self.token.end = self.position;
        self.token.raw_text = &self.input[self.token.start..self.token.end];
        self.token.text = self.token.raw_text.into();
        Ok(())
    }

    fn finish_token_with<F>(&mut self, kind: TokenKind, modifier: F) -> Result<(), ScanError>
    where
        F: Fn(&mut Token) -> Result<(), ScanError>,
    {
        self.token.kind = kind;
        self.token.end = self.position;
        self.token.raw_text = &self.input[self.token.start..self.token.end];
        self.token.text = self.token.raw_text.into();
        modifier(&mut self.token)
    }

    fn scan_identifier(&mut self) -> Result<(), ScanError> {
        self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.scan_char()?;
                }
                _ => {
                    return self.finish_token(TokenKind::Identifier);
                }
            }
        }
        self.finish_token(TokenKind::Identifier)
    }

    fn scan_number(&mut self) -> Result<(), ScanError> {
        self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                '0'..='9' | '_' => {
                    self.scan_char()?;
                }
                _ => {
                    return self.finish_token(TokenKind::Number);
                }
            }
        }
        fn cleanup_number(token: &mut Token) -> Result<(), ScanError> {
            let s = token
                .raw_text
                .chars()
                .filter(|c| matches!(*c, '0'..='9'))
                .collect::<String>();
            token.text = s.into();
            Ok(())
        }
        self.finish_token_with(TokenKind::Number, cleanup_number)
    }

    fn single_symbol(&mut self, symbol: Symbol) -> Result<(), ScanError> {
        self.scan_char()?;
        self.finish_token(TokenKind::Symbol(symbol))
    }

    fn maybe_double_symbol(
        &mut self,
        expected: char,
        single_symbol: Symbol,
        double_symbol: Symbol,
    ) -> Result<(), ScanError> {
        self.scan_char()?;
        match self.current_char {
            Some(ch) if ch == expected => return self.single_symbol(double_symbol),

            _ => return self.finish_token(TokenKind::Symbol(single_symbol)),
        }
    }

    fn scan_string(&mut self) -> Result<(), ScanError> {
        self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    self.scan_char()?;
                    return self.finish_token(TokenKind::String);
                }
                '\\' => {
                    self.scan_char()?;
                    match self.current_char {
                        Some(ch) if "nrtb".contains(ch) => {
                        self.scan_char()?
                        }
                        Some(_) => return Err(ScanError::UnexpectedCharacterInEscapeSequence{offset: self.position, unexpected: ch}),
                        None => return Err(ScanError::UnexpectedEndOfInputInEscapeSequence{offset: self.position}),
                    }
                }
                _ => {
                    self.scan_char()?;
                }
            }
        }
        Err(ScanError::UnexpectedEndOfInputInString {
            offset: self.position,
            string_start: self.token.start,
        })
    }

    pub fn scan(&mut self) -> Result<(), ScanError> {
        self.skip_whitespace()?;
        self.token.start = self.position;
        if let Some(ch) = self.current_char {
            match ch {
                'a'..='z' | 'A'..='Z' | '_' => return self.scan_identifier(),
                '0'..='9' => return self.scan_number(),
                ':' => return self.maybe_double_symbol(':', Symbol::Colon, Symbol::DoubleColon),
                '=' => return self.maybe_double_symbol('=', Symbol::Eq, Symbol::EqEq),
                ';' => return self.single_symbol(Symbol::Semicolon),
                ',' => return self.single_symbol(Symbol::Comma),
                '"' => return self.scan_string(),
                _ => {
                    return Err(ScanError::UnexpectedCharacter {
                        offset: self.position,
                        unexpected: ch,
                    })
                }
            }
        } else {
            return self.finish_token(TokenKind::Eof);
        }
    }

    pub fn token(&self) -> &Token<'src> {
        &self.token
    }
}
