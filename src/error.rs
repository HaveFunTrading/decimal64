//! Error types returned by decimal parsing and conversion operations.

use thiserror::Error;

/// Error returned when an output buffer cannot hold a formatted decimal.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
#[error("provided buffer length {provided} is too small, requires at least {required} bytes")]
pub struct BufferTooSmall {
    /// Number of bytes required for this value.
    pub required: usize,
    /// Number of bytes provided by the caller.
    pub provided: usize,
}

/// An error returned by a decimal operation.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// The input cannot be represented as a decimal.
    #[error("invalid input: {0}")]
    InvalidInput(InvalidInputKind),
    /// The result exceeds the `u64` representation.
    #[error("overflow")]
    Overflow,
}

/// The reason an input is invalid.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum InvalidInputKind {
    /// The input is empty.
    #[error("empty input")]
    Empty,
    /// The input contains no digits.
    #[error("input contains no digits")]
    MissingDigits,
    /// The input contains a byte that is not valid decimal syntax.
    #[error("invalid byte in input: {0:#04x}")]
    InvalidByte(u8),
    /// The input contains more than one decimal point.
    #[error("multiple decimal points")]
    MultipleDecimalPoints,
    /// The value has nonzero digits beyond the target scale.
    #[error("value has more precision than the target scale")]
    PrecisionLoss,
    /// The input represents a negative number.
    #[error("negative number")]
    NegativeNumber,
    /// The input is an infinite floating-point value.
    #[error("infinite number")]
    InfiniteNumber,
    /// The input is a NaN floating-point value.
    #[error("not a number")]
    NotANumber,
}
