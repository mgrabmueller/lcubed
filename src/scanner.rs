#[derive(Debug)]
#[allow(dead_code)]
pub enum ScanError {
    UnexpectedEndOfInput(usize),
    UnexpectedCharacter(usize, char),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedEndOfInput(ofs) => write!(f, "unexpected end of input at offset {}", ofs),
            ScanError::UnexpectedCharacter(ofs, c) => write!(f, "unexpected character {:?} at offset {}", c, ofs),
        }
    }
}