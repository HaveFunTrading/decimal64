//! Fixed-point arithmetic for [`crate::DecimalI64`].

use crate::{DecimalI64, ScaleMetrics};

impl_decimal_arithmetic!(DecimalI64, i64);
impl_decimal_operators!(DecimalI64);

impl<S: ScaleMetrics> DecimalI64<S> {
    /// Returns the absolute value.
    ///
    /// # Panics
    ///
    /// Panics if `self` is [`Self::MIN`].
    #[must_use]
    #[inline]
    pub const fn abs(self) -> Self {
        match self.checked_abs() {
            Some(value) => value,
            None => panic!("attempt to calculate absolute value with overflow"),
        }
    }

    /// Checked negation.
    #[must_use]
    #[inline]
    pub const fn checked_neg(self) -> Option<Self> {
        match self.0.checked_neg() {
            Some(value) => Some(Self::new(value)),
            None => None,
        }
    }

    /// Checked absolute value.
    #[must_use]
    #[inline]
    pub const fn checked_abs(self) -> Option<Self> {
        match self.0.checked_abs() {
            Some(value) => Some(Self::new(value)),
            None => None,
        }
    }

    /// Saturating negation.
    #[must_use]
    #[inline]
    pub const fn saturating_neg(self) -> Self {
        Self::new(self.0.saturating_neg())
    }

    /// Saturating absolute value.
    #[must_use]
    #[inline]
    pub const fn saturating_abs(self) -> Self {
        Self::new(self.0.saturating_abs())
    }

    /// Wrapping negation.
    #[must_use]
    #[inline]
    pub const fn wrapping_neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }

    /// Wrapping absolute value.
    #[must_use]
    #[inline]
    pub const fn wrapping_abs(self) -> Self {
        Self::new(self.0.wrapping_abs())
    }

    /// Overflowing negation.
    #[must_use]
    #[inline]
    pub const fn overflowing_neg(self) -> (Self, bool) {
        let (value, overflowed) = self.0.overflowing_neg();
        (Self::new(value), overflowed)
    }

    /// Overflowing absolute value.
    #[must_use]
    #[inline]
    pub const fn overflowing_abs(self) -> (Self, bool) {
        let (value, overflowed) = self.0.overflowing_abs();
        (Self::new(value), overflowed)
    }

    #[inline]
    const fn wide_mul(self, rhs: Self) -> i128 {
        (self.0 as i128 * rhs.0 as i128) / S::SCALE_FACTOR as i128
    }

    #[inline]
    const fn wide_div(self, rhs: Self) -> i128 {
        (self.0 as i128 * S::SCALE_FACTOR as i128) / rhs.0 as i128
    }

    #[inline]
    const fn from_wide(value: i128) -> Option<Self> {
        if value < i64::MIN as i128 || value > i64::MAX as i128 {
            None
        } else {
            Some(Self::new(value as i64))
        }
    }

    #[inline]
    const fn saturating_from_wide(value: i128) -> Self {
        if value < i64::MIN as i128 {
            Self::MIN
        } else if value > i64::MAX as i128 {
            Self::MAX
        } else {
            Self::new(value as i64)
        }
    }
}

impl<S: ScaleMetrics> std::ops::Neg for DecimalI64<S> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.checked_neg().expect("attempt to negate with overflow")
    }
}
