use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(InvalidInputKind),
    #[error("overflow")]
    Overflow,
}

#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum InvalidInputKind {
    #[error("invalid character in input: {0}")]
    InvalidCharacter(char),
    #[error("negative number")]
    NegativeNumber,
    #[error("infinite number")]
    InfiniteNumber,
    #[error("value less than one")]
    LessThanOne,
}
