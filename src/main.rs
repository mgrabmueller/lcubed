use error::Error;
use token::{Symbol, Token, TokenKind};
use scanner::ScanError;

mod error;
mod token;
mod scanner;

fn main() -> Result<(), Error> {
    println!("Hello, world!");
    println!("{:?}", Token::new(TokenKind::Symbol(Symbol::Comma)));
    Err(ScanError::UnexpectedEndOfInput(12))?;
    println!("Goodbye, world!");
    Ok(())
}
