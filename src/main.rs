use error::Error;

mod error;

fn main() -> Result<(), Error> {
    println!("Hello, world!");
    Err(Error::Other("foo".into()))?;
    println!("Goodbye, world!");
    Ok(())
}
