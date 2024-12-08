use std::{borrow::Cow, str::CharIndices};

use crate::token::{Keyword, Symbol, Token, TokenKind};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ScanError {
    UnexpectedEndOfInput { offset: usize },
    UnexpectedCharacter { offset: usize, unexpected: char },
    UnexpectedCharacterInEscapeSequence { offset: usize, unexpected: char },
    UnexpectedEndOfInputInString { offset: usize, string_start: usize },
    UnexpectedEndOfInputInEscapeSequence { offset: usize },
}

impl std::error::Error for ScanError {}

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
                write!(
                    f,
                    "unexpected character {unexpected:?} in escape sequence at offset {offset}"
                )
            }
            ScanError::UnexpectedEndOfInputInEscapeSequence { offset } => {
                write!(
                    f,
                    "unexpected end of input at offset {offset} in escape sequence"
                )
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
    /// Create a new scanner that will tokenize the given string.
    ///
    /// # Errors
    /// Returns an error if the string does not start with a valid token.
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

    /// Move the scanner to the next character.
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

    /// Move the scanner to the next non-whitespace character.
    fn skip_whitespace(&mut self) -> Result<(), ScanError> {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() {
                break;
            }
            self.scan_char()?;
        }
        Ok(())
    }

    /// Set the kind and end position, and the text/raw text fields of
    /// the token to the scanned porition of the input.
    fn finish_token(&mut self, kind: TokenKind) -> Result<(), ScanError> {
        self.token.kind = kind;
        self.token.end = self.position;
        self.token.raw_text = &self.input[self.token.start..self.token.end];
        self.token.text = self.token.raw_text.into();
        Ok(())
    }

    /// Set the kind and end position, and the text/raw text fields of
    /// the token to the scanned porition of the input. Before returning, calls
    /// the given modifier function on the token, which might do some
    /// post-processing.
    fn finish_token_with<F>(&mut self, kind: TokenKind, modifier: F) -> Result<(), ScanError>
    where
        F: Fn(&mut Token) -> Result<(), ScanError>,
    {
        self.finish_token(kind)?;
        modifier(&mut self.token)
    }

    fn scan_identifier_or_keyword(&mut self) -> Result<(), ScanError> {
        let finish = |scanner: &mut Scanner| -> Result<(), ScanError> {
            scanner.finish_token(TokenKind::Identifier)?;
            if let Some(kw) = match scanner.token.raw_text {
                "if" => Some(Keyword::If),
                "else" => Some(Keyword::Else),
                "end" => Some(Keyword::End),
                "fun" => Some(Keyword::Fun),
                _ => None,
            } {
                scanner.token.kind = TokenKind::Keyword(kw);
            }
            Ok(())
        };
        self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.scan_char()?;
                }
                _ => {
                    return finish(self);
                }
            }
        }
        self.finish_token(TokenKind::Identifier)
    }

    fn scan_number(&mut self) -> Result<(), ScanError> {
        fn cleanup_number(token: &mut Token) -> Result<(), ScanError> {
            let s = token
                .raw_text
                .chars()
                .filter(|c| matches!(*c, '0'..='9'))
                .collect::<String>();
            token.text = s.into();
            Ok(())
        }
                self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                '0'..='9' | '_' => {
                    self.scan_char()?;
                }
                _ => {
                    return self.finish_token_with(TokenKind::Number, cleanup_number);
                }
            }
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

    fn current_text(&self) -> &'src str {
        &self.input[self.token.start..self.position]
    }

    fn scan_string(&mut self) -> Result<(), ScanError> {
        let mut clean_string = None;
        self.scan_char()?;
        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    self.scan_char()?;
                    let text = if let Some(cs) = clean_string {
                        Cow::from(cs)
                    } else {
                        let ct = self.current_text();
                        // UTF-8 length of character '"' is always 1, so this trims
                        // off the quotes at both ends.
                        Cow::from(&ct[1..ct.len() - 1])
                    };
                    // return self.finish_token_with_text(TokenKind::String, Some(text));
                    self.finish_token(TokenKind::String)?;
                    self.token.text = text;
                    return Ok(());
                }
                '\\' => {
                    self.scan_char()?;
                    match self.current_char {
                        Some(ch) if "nrt\\\"'".contains(ch) => {
                            let mut s = match clean_string.take() {
                                None => {
                                    let ct = self.current_text();
                                    // trim off quote at the start and the backslash that 
                                    // introduced the current escape sequence.
                                    ct[1..ct.len() - 1].to_string()
                                }
                                Some(s) => s,
                            };
                            match ch {
                                'n' => s.push('\n'),
                                'r' => s.push('\r'),
                                't' => s.push('\t'),
                                '\\' => s.push('\\'),
                                '"' => s.push('"'),
                                '\'' => s.push('\''),
                                _ => unreachable!(),
                            }
                            clean_string = Some(s);
                            self.scan_char()?
                        }
                        Some(ch) => {
                            return Err(ScanError::UnexpectedCharacterInEscapeSequence {
                                offset: self.position,
                                unexpected: ch,
                            })
                        }
                        None => {
                            return Err(ScanError::UnexpectedEndOfInputInEscapeSequence {
                                offset: self.position,
                            })
                        }
                    }
                }
                _ => {
                    if let Some(s) = &mut clean_string {
                        s.push(ch);
                    }
                    self.scan_char()?;
                }
            }
        }
        Err(ScanError::UnexpectedEndOfInputInString {
            offset: self.position,
            string_start: self.token.start,
        })
    }

    fn skip_line_comment(&mut self) -> Result<(), ScanError> {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                return self.scan_char();
            }
            self.scan_char()?;
        }
        Ok(())
    }

    /// Advance the scanner to the next token, skipping over whitespace and comments.
    pub fn scan(&mut self) -> Result<(), ScanError> {
        loop {
            self.skip_whitespace()?;
            self.token.start = self.position;
            if let Some(ch) = self.current_char {
                match ch {
                    '/' => {
                        self.scan_char()?;
                        match self.current_char {
                            Some('/') => self.skip_line_comment()?,
                            _ => return self.finish_token(TokenKind::Symbol(Symbol::Slash)),
                        }
                    }
                    'a'..='z' | 'A'..='Z' | '_' => return self.scan_identifier_or_keyword(),
                    '0'..='9' => return self.scan_number(),
                    ':' => {
                        return self.maybe_double_symbol(':', Symbol::Colon, Symbol::DoubleColon)
                    }
                    '=' => return self.maybe_double_symbol('=', Symbol::Eq, Symbol::EqEq),
                    ';' => return self.single_symbol(Symbol::Semicolon),
                    ',' => return self.single_symbol(Symbol::Comma),
                    '.' => return self.single_symbol(Symbol::Dot),
                    '+' => return self.single_symbol(Symbol::Plus),
                    '*' => return self.single_symbol(Symbol::Star),
                    '-' => return self.maybe_double_symbol('>', Symbol::Minus, Symbol::Arrow),
                    '\\' => return self.single_symbol(Symbol::Backslash),
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
    }

    pub fn token(&self) -> &Token<'src> {
        &self.token
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn run(input: &str) -> Result<Vec<Token>, ScanError> {
        let mut scanner = Scanner::new(input)?;
        let mut output = Vec::new();
        loop {
            output.push(scanner.token().clone());
            if scanner.token().kind() == TokenKind::Eof {
                break;
            }
            scanner.scan()?;
        }
        Ok(output)
    }

    #[test]
    fn whitespace() {
        let ts = run("\t\n\rx").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "x");
        assert_eq!(ts[0].raw_text(), "x");
        assert_eq!(ts[0].start(), 3);
        assert_eq!(ts[0].end(), 4);

        assert_eq!(ts[1].kind(), TokenKind::Eof);
        assert_eq!(ts[1].text(), "");
        assert_eq!(ts[1].raw_text(), "");
        assert_eq!(ts[1].start(), 4);
        assert_eq!(ts[1].end(), 4);
    }

    #[test]
    fn numbers() {
        let ts = run("1").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "1");
        assert_eq!(ts[0].raw_text(), "1");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 1);

        let ts = run("1000").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "1000");
        assert_eq!(ts[0].raw_text(), "1000");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 4);

        let ts = run("9999").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "9999");
        assert_eq!(ts[0].raw_text(), "9999");

        let ts = run("1_000").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "1000");
        assert_eq!(ts[0].raw_text(), "1_000");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 5);

        let ts = run("1_000_000_000_000_000").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "1000000000000000");
        assert_eq!(ts[0].raw_text(), "1_000_000_000_000_000");

        // Ensure number is parsed correctly if not at end of input.
        let ts = run("1_000  x").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Number);
        assert_eq!(ts[0].text(), "1000");
        assert_eq!(ts[0].raw_text(), "1_000");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 5);
    }

    #[test]
    fn identifiers() {
        let ts = run("a").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "a");
        assert_eq!(ts[0].raw_text(), "a");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 1);

        let ts = run("abc").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "abc");
        assert_eq!(ts[0].raw_text(), "abc");

        let ts = run("_").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "_");
        assert_eq!(ts[0].raw_text(), "_");

        let ts = run("a_1").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "a_1");
        assert_eq!(ts[0].raw_text(), "a_1");

        let ts = run("_1").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "_1");
        assert_eq!(ts[0].raw_text(), "_1");

        let ts =
            run("asdfadsdflHJLHLadfJHJH__AS777SDHJ456789LH_1").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "asdfadsdflHJLHLadfJHJH__AS777SDHJ456789LH_1");
        assert_eq!(
            ts[0].raw_text(),
            "asdfadsdflHJLHLadfJHJH__AS777SDHJ456789LH_1"
        );
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 43);

        // Ensure correct scanning if not at end of input.
        let ts = run("a_1 x").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "a_1");
        assert_eq!(ts[0].raw_text(), "a_1");
    }

    #[test]
    fn symbols() {
        let ts = run("; :: : = == , \\").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Symbol(Symbol::Semicolon));
        assert_eq!(ts[0].text(), ";");
        assert_eq!(ts[0].raw_text(), ";");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 1);

        assert_eq!(ts[1].kind(), TokenKind::Symbol(Symbol::DoubleColon));
        assert_eq!(ts[1].text(), "::");
        assert_eq!(ts[1].raw_text(), "::");
        assert_eq!(ts[1].start(), 2);
        assert_eq!(ts[1].end(), 4);

        assert_eq!(ts[2].kind(), TokenKind::Symbol(Symbol::Colon));
        assert_eq!(ts[2].text(), ":");
        assert_eq!(ts[2].raw_text(), ":");
        assert_eq!(ts[2].start(), 5);
        assert_eq!(ts[2].end(), 6);

        assert_eq!(ts[3].kind(), TokenKind::Symbol(Symbol::Eq));
        assert_eq!(ts[3].text(), "=");
        assert_eq!(ts[3].raw_text(), "=");
        assert_eq!(ts[3].start(), 7);
        assert_eq!(ts[3].end(), 8);

        assert_eq!(ts[4].kind(), TokenKind::Symbol(Symbol::EqEq));
        assert_eq!(ts[5].kind(), TokenKind::Symbol(Symbol::Comma));
        assert_eq!(ts[6].kind(), TokenKind::Symbol(Symbol::Backslash));
    }

    #[test]
    fn keywords() {
        let ts = run("if end else fun ifthen funny").expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::Keyword(Keyword::If));
        assert_eq!(ts[0].text(), "if");
        assert_eq!(ts[0].raw_text(), "if");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 2);

        assert_eq!(ts[1].kind(), TokenKind::Keyword(Keyword::End));
        assert_eq!(ts[1].text(), "end");
        assert_eq!(ts[1].raw_text(), "end");
        assert_eq!(ts[1].start(), 3);
        assert_eq!(ts[1].end(), 6);

        assert_eq!(ts[2].kind(), TokenKind::Keyword(Keyword::Else));
        assert_eq!(ts[2].text(), "else");
        assert_eq!(ts[2].raw_text(), "else");
        assert_eq!(ts[2].start(), 7);
        assert_eq!(ts[2].end(), 11);

        assert_eq!(ts[3].kind(), TokenKind::Keyword(Keyword::Fun));
        assert_eq!(ts[3].text(), "fun");
        assert_eq!(ts[3].raw_text(), "fun");
        assert_eq!(ts[3].start(), 12);
        assert_eq!(ts[3].end(), 15);

        assert_eq!(ts[4].kind(), TokenKind::Identifier);
        assert_eq!(ts[4].text(), "ifthen");
        assert_eq!(ts[4].raw_text(), "ifthen");
        assert_eq!(ts[4].start(), 16);
        assert_eq!(ts[4].end(), 22);

        assert_eq!(ts[5].kind(), TokenKind::Identifier);
        assert_eq!(ts[5].text(), "funny");
        assert_eq!(ts[5].raw_text(), "funny");
        assert_eq!(ts[5].start(), 23);
        assert_eq!(ts[5].end(), 28);
    }

    #[test]
    fn strings() {
        let ts = run(r###""hello" "" "\r" "\\""###).expect("scanning example input");
        assert_eq!(ts[0].kind(), TokenKind::String);
        assert_eq!(ts[0].text(), "hello");
        assert_eq!(ts[0].raw_text(), "\"hello\"");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 7);

        assert_eq!(ts[1].kind(), TokenKind::String);
        assert_eq!(ts[1].text(), "");
        assert_eq!(ts[1].raw_text(), "\"\"");
        assert_eq!(ts[1].start(), 8);
        assert_eq!(ts[1].end(), 10);

        assert_eq!(ts[2].kind(), TokenKind::String);
        assert_eq!(ts[2].text(), "\r");
        assert_eq!(ts[2].raw_text(), "\"\\r\"");
        assert_eq!(ts[2].start(), 11);
        assert_eq!(ts[2].end(), 15);

        assert_eq!(ts[3].kind(), TokenKind::String);
        assert_eq!(ts[3].text(), "\\");
        assert_eq!(ts[3].raw_text(), "\"\\\\\"");
        assert_eq!(ts[3].start(), 16);
        assert_eq!(ts[3].end(), 20);
    }

    #[test]
    fn strings_errors() {
        let e = run(r#"""#).expect_err("should fail");
        assert!(matches!(e, ScanError::UnexpectedEndOfInputInString { string_start: 0, offset: 1 }));
        let e = run(r#""H\ello""#).expect_err("should fail");
        assert!(matches!(e, ScanError::UnexpectedCharacterInEscapeSequence { offset: 3, unexpected: 'e' }));
        let e = run(r#""H\"#).expect_err("should fail");
        assert!(matches!(e, ScanError::UnexpectedEndOfInputInEscapeSequence { offset: 3 }));
    }

    #[test]
    fn comments() {
        let ts = run(r###"hello
        // line comment
        world
        // another one at the end (no newline)"###)
        .expect("scanning example input");

        assert_eq!(ts[0].kind(), TokenKind::Identifier);
        assert_eq!(ts[0].text(), "hello");
        assert_eq!(ts[0].raw_text(), "hello");
        assert_eq!(ts[0].start(), 0);
        assert_eq!(ts[0].end(), 5);

        assert_eq!(ts[1].kind(), TokenKind::Identifier);
        assert_eq!(ts[1].text(), "world");
        assert_eq!(ts[1].raw_text(), "world");
        assert_eq!(ts[1].start(), 38);
        assert_eq!(ts[1].end(), 43);
    }
}
