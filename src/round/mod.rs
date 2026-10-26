//! Rounding decimal values to tick-size multiples.
//!
//! [`HalfUp`], [`Floor`], and [`Ceil`] provide the built-in rounding
//! strategies. Apply one with [`crate::DecimalU64::round`] or
//! [`crate::DecimalI64::round`]. Values and tick sizes
//! must have the same scale.
//!
//! ```
//! use decimal64::round::HalfUp;
//! use decimal64::{DecimalU64, U2};
//!
//! let value = DecimalU64::<U2>::from_str("1.26").unwrap();
//! let tick_size = DecimalU64::<U2>::from_str("0.10").unwrap();
//! assert_eq!("1.30", value.round::<HalfUp>(tick_size).to_string());
//! ```

mod signed;
mod unsigned;

use crate::{DecimalI64, DecimalU64};

impl_decimal_round!(DecimalU64);
impl_decimal_round!(DecimalI64);

/// A strategy for rounding a decimal to a multiple of a tick size.
///
/// Implementations receive values with the same scale and return a value at
/// that scale. The built-in policies require a nonzero unsigned tick or a
/// strictly positive signed tick and panic if the result is not representable.
pub trait RoundingPolicy<D> {
    /// Rounds `value` to `tick_size`.
    ///
    /// # Panics
    ///
    /// Implementations may panic when `tick_size` is zero or the rounded value
    /// is not representable. See the policy's documentation for its contract.
    fn round(value: D, tick_size: D) -> D;
}

/// Rounds to the nearest tick, choosing the result away from zero at a
/// halfway point.
///
/// For example, `0.125` rounded to a `0.01` tick is `0.13`.
///
/// # Panics
///
/// Panics if the tick size is zero or the rounded result cannot be represented.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HalfUp;

/// Rounds down to the nearest tick.
///
/// For example, `0.129` rounded to a `0.01` tick is `0.12`.
///
/// # Panics
///
/// Panics if the tick size is zero.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Floor;

/// Rounds up to the nearest tick unless the value is already a multiple.
///
/// For example, `0.121` rounded to a `0.01` tick is `0.13`.
///
/// # Panics
///
/// Panics if the tick size is zero or the rounded result cannot be represented.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ceil;
