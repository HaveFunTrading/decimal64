#![warn(missing_docs)]

//! Fast fixed-point decimal arithmetic backed by a single [`u64`] or [`i64`].
//!
//! [`DecimalU64`] and [`DecimalI64`] store an unscaled integer while their
//! decimal scale is encoded in the type. The crate provides the sealed scales
//! [`U0`] through [`U8`], so values with different scales are different Rust
//! types and cannot be mixed accidentally.
//!
//! # Example
//!
//! ```
//! use decimal64::{DecimalU64, U2, U8};
//!
//! let price = DecimalU64::<U2>::from_str("80500.32")?;
//! let quantity = DecimalU64::<U2>::from_str("2.00")?;
//! let total = price.checked_mul(quantity).unwrap();
//! assert_eq!("161000.64", total.to_string());
//!
//! let normalized = price.rescale::<U8>()?;
//! assert_eq!("80500.32000000", normalized.to_string());
//! # Ok::<(), decimal64::Error>(())
//! ```
//!
//! # Parsing and scaling
//!
//! [`DecimalU64::from_str`] and [`DecimalU64::from_slice`] accept plain,
//! non-negative decimal notation; the corresponding [`DecimalI64`] methods
//! also accept a leading minus sign. Parsing is exact: fractional digits beyond
//! the target scale are accepted only when they are zero, and parsing never
//! rounds. Scientific notation is intentionally unsupported.
//!
//! The `rescale` methods convert between the built-in scales. Upscaling returns
//! [`Error::Overflow`] when the result does not fit; downscaling rounds halfway
//! values away from zero (equivalent to half-up for unsigned values).
//!
//! # Arithmetic
//!
//! Arithmetic operands must have the same scale. The standard `+`, `-`, `*`,
//! and `/` operators panic on overflow, underflow, or division by zero in every
//! build profile. The corresponding `checked_*`, `saturating_*`, `wrapping_*`,
//! and `overflowing_*` method families provide explicit alternatives.
//! Multiplication and division truncate toward zero at the result scale.
//!
//! Values with different scales cannot be combined accidentally:
//!
//! ```compile_fail
//! use decimal64::{DecimalU64, U2, U4};
//!
//! let price = DecimalU64::<U2>::new(100);
//! let quantity = DecimalU64::<U4>::new(100);
//! let _ = price + quantity;
//! ```
//!
//! # Formatting
//!
//! Both decimal types can write directly into caller-provided buffers without
//! allocation using `write_to` and `write_to_trimmed`. [`std::fmt::Display`] and
//! `ToString::to_string` are also supported.
//!
//! # Optional features
//!
//! The `serde` feature serializes and deserializes all built-in scales as
//! decimal strings. Numeric tokens are rejected so deserialization follows the
//! same exact policy as string parsing.

#[macro_use]
mod macros;

mod arithmetic;
mod convert;
mod decimal_i64;
mod decimal_u64;
pub mod error;
mod format;
mod parse;
pub mod round;
mod scale;
#[cfg(feature = "serde")]
mod serde;

pub use decimal_i64::DecimalI64;
pub use decimal_u64::DecimalU64;
pub use error::{BufferTooSmall, Error, InvalidInputKind};
pub use scale::{ScaleMetrics, U0, U1, U2, U3, U4, U5, U6, U7, U8};
