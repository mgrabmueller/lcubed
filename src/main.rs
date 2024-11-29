use error::Error;
use parser::Parser;

mod error;
mod scanner;
mod token;
mod parser;
mod ast;

fn main() -> Result<(), Error> {
    let input = "main :: Integer; main = 2;";
    let mut parser = Parser::new(input)?;
    let _ = parser.parse_program()?;
    println!("Parse OK!");
    Ok(())
}
