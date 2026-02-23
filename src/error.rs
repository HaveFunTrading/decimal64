use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(InvalidInputKind),
    #[error("overflow")]
    Overflow,
    #[error("precision loss: {from_scale} -> {to_scale}")]
    PrecisionLoss { from_scale: u8, to_scale: u8 },
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
