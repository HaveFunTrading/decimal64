use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid character in input: {0}")]
    InvalidCharacterInput(char),
    #[error("overflow: {0}")]
    Overflow(String),
}
