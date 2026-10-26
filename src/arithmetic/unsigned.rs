//! Fixed-point arithmetic for [`DecimalU64`].
//!
//! Both operands must have the same scale, and every result retains that
//! scale. Addition and subtraction operate directly on the raw representation.
//! Multiplication computes `(lhs.raw * rhs.raw) / scale_factor`, while division
//! computes `(lhs.raw * scale_factor) / rhs.raw`; both truncate toward zero.
//!
//! The `+`, `-`, `*`, and `/` operators panic on arithmetic overflow,
//! underflow, or division by zero in every build profile. Use the corresponding
//! `checked_*`, `saturating_*`, `wrapping_*`, or `overflowing_*` methods when a
//! different policy is required. Of the division methods, only
//! [`DecimalU64::checked_div`] handles a zero divisor without panicking.
//!
//! ```
//! use decimal64::{DecimalU64, U2};
//!
//! let price = DecimalU64::<U2>::from_str("12.50").unwrap();
//! let quantity = DecimalU64::<U2>::from_str("2.00").unwrap();
//!
//! assert_eq!("25.00", price.checked_mul(quantity).unwrap().to_string());
//! assert_eq!(None, DecimalU64::<U2>::MAX.checked_add(DecimalU64::ONE));
//! assert_eq!(DecimalU64::<U2>::MAX, DecimalU64::<U2>::MAX.saturating_add(DecimalU64::ONE));
//! ```

use crate::{DecimalU64, ScaleMetrics};

impl_decimal_arithmetic!(DecimalU64, u64);
impl_decimal_operators!(DecimalU64);

impl<S: ScaleMetrics> DecimalU64<S> {
    #[inline]
    const fn wide_mul(self, rhs: Self) -> u128 {
        (self.0 as u128 * rhs.0 as u128) / S::SCALE_FACTOR as u128
    }

    #[inline]
    const fn wide_div(self, rhs: Self) -> u128 {
        (self.0 as u128 * S::SCALE_FACTOR as u128) / rhs.0 as u128
    }

    #[inline]
    const fn from_wide(value: u128) -> Option<Self> {
        if value > u64::MAX as u128 {
            None
        } else {
            Some(Self::new(value as u64))
        }
    }

    #[inline]
    const fn saturating_from_wide(value: u128) -> Self {
        if value > u64::MAX as u128 {
            Self::MAX
        } else {
            Self::new(value as u64)
        }
    }
}
