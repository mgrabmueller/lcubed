#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Other(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Other(s) => s.fmt(f),
        }
    }
}
