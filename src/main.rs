use error::Error;
use scanner::Scanner;
use token::TokenKind;

mod error;
mod scanner;
mod token;

fn main() -> Result<(), Error> {
    let input = r#"main :: Integer;
     main = 1_000_000;
     test :: String;
     test = "hallo world";
     id :: X -> X;
     id = \ x. x;
     before
     // blub.
     middle // blob
     after
     "#;
    let mut scanner = Scanner::new(input)?;
    loop {
        println!("{:?}", scanner.token());
        if scanner.token().kind() == TokenKind::Eof {
            break;
        }
        scanner.scan()?;
    }
    Ok(())
}
