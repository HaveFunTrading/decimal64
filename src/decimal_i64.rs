//! Signed fixed-point decimal values.

use crate::{DecimalU64, ScaleMetrics};
use std::marker::PhantomData;

/// A signed fixed-point decimal with a compile-time scale.
///
/// `S` is one of the built-in [`crate::U0`] through [`crate::U8`] scale types.
/// The raw unscaled representation is available through the public tuple field.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DecimalI64<S>(
    /// Raw unscaled integer representation.
    pub i64,
    PhantomData<S>,
);

impl_decimal_core!(DecimalI64, i64, i64::MIN, i64::MAX);

impl<S: ScaleMetrics> DecimalI64<S> {
    /// Returns `true` if this decimal is positive.
    #[must_use]
    #[inline]
    pub const fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    /// Returns `true` if this decimal is negative.
    #[must_use]
    #[inline]
    pub const fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    /// Returns the magnitude as an unsigned decimal without overflow.
    #[must_use]
    #[inline]
    pub const fn unsigned_abs(self) -> DecimalU64<S> {
        DecimalU64::new(self.0.unsigned_abs())
    }
}
