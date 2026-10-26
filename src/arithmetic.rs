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
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

impl<S: ScaleMetrics> Mul for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs).expect("attempt to multiply with overflow")
    }
}

impl<S: ScaleMetrics> Add for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).expect("attempt to add with overflow")
    }
}

impl<S: ScaleMetrics> Sub for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).expect("attempt to subtract with overflow")
    }
}

impl<S: ScaleMetrics> Div for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        self.checked_div(rhs).expect("attempt to divide with overflow")
    }
}

impl<S: ScaleMetrics> AddAssign for DecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<S: ScaleMetrics> AddAssign<&DecimalU64<S>> for DecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        *self += *rhs;
    }
}

impl<S: ScaleMetrics> SubAssign for DecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<S: ScaleMetrics> SubAssign<&DecimalU64<S>> for DecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: &Self) {
        *self -= *rhs;
    }
}

impl<S: ScaleMetrics> MulAssign for DecimalU64<S> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<S: ScaleMetrics> MulAssign<&DecimalU64<S>> for DecimalU64<S> {
    #[inline]
    fn mul_assign(&mut self, rhs: &Self) {
        *self *= *rhs;
    }
}

impl<S: ScaleMetrics> DivAssign for DecimalU64<S> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<S: ScaleMetrics> DivAssign<&DecimalU64<S>> for DecimalU64<S> {
    #[inline]
    fn div_assign(&mut self, rhs: &Self) {
        *self /= *rhs;
    }
}

impl<S: ScaleMetrics> Sum for DecimalU64<S> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Self::ZERO;
        for i in iter {
            sum += i;
        }
        sum
    }
}

impl<'a, S: ScaleMetrics> Sum<&'a DecimalU64<S>> for DecimalU64<S> {
    fn sum<I: Iterator<Item = &'a DecimalU64<S>>>(iter: I) -> Self {
        let mut sum = Self::ZERO;
        for i in iter {
            sum += i;
        }
        sum
    }
}

impl<S: ScaleMetrics> Product for DecimalU64<S> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut product = Self::ONE;
        for value in iter {
            product *= value;
        }
        product
    }
}

impl<'a, S: ScaleMetrics> Product<&'a DecimalU64<S>> for DecimalU64<S> {
    fn product<I: Iterator<Item = &'a DecimalU64<S>>>(iter: I) -> Self {
        let mut product = Self::ONE;
        for value in iter {
            product *= value;
        }
        product
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Checked fixed-point addition.
    ///
    /// Returns `None` if the result cannot be represented by `Self`.
    #[must_use]
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(sum) => Some(Self::new(sum)),
            None => None,
        }
    }

    /// Checked fixed-point subtraction.
    ///
    /// Returns `None` if the result would be negative.
    #[must_use]
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(diff) => Some(Self::new(diff)),
            None => None,
        }
    }

    /// Checked fixed-point multiplication.
    ///
    /// The result is truncated toward zero at the scale of `Self`. Returns `None` if
    /// the scaled result cannot be represented by `Self`.
    #[must_use]
    #[inline]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.wide_mul(rhs);
        if result > u64::MAX as u128 {
            None
        } else {
            Some(Self::new(result as u64))
        }
    }

    /// Checked fixed-point division.
    ///
    /// The result is truncated toward zero at the scale of `Self`. Returns `None` if
    /// `rhs` is zero or if the scaled result cannot be represented by `Self`.
    #[must_use]
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let result = self.wide_div(rhs);
        if result > u64::MAX as u128 {
            None
        } else {
            Some(Self::new(result as u64))
        }
    }

    /// Saturating fixed-point addition.
    ///
    /// Returns [`Self::MAX`] if the mathematical result is too large.
    #[must_use]
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::new(self.0.saturating_add(rhs.0))
    }

    /// Saturating fixed-point subtraction.
    ///
    /// Returns [`Self::ZERO`] if the mathematical result would be negative.
    #[must_use]
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self::new(self.0.saturating_sub(rhs.0))
    }

    /// Saturating fixed-point multiplication.
    ///
    /// The result is truncated toward zero at the scale of `Self` and clamped to
    /// [`Self::MAX`] if it is too large.
    #[must_use]
    #[inline]
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        let result = self.wide_mul(rhs);
        if result > u64::MAX as u128 {
            Self::MAX
        } else {
            Self::new(result as u64)
        }
    }

    /// Saturating fixed-point division.
    ///
    /// The result is truncated toward zero at the scale of `Self` and clamped to
    /// [`Self::MAX`] if it is too large.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    #[must_use]
    #[inline]
    pub const fn saturating_div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let result = self.wide_div(rhs);
        if result > u64::MAX as u128 {
            Self::MAX
        } else {
            Self::new(result as u64)
        }
    }

    /// Wrapping fixed-point addition.
    ///
    /// Wraps the raw result modulo `2^64`.
    #[must_use]
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_add(rhs.0))
    }

    /// Wrapping fixed-point subtraction.
    ///
    /// Wraps the raw result modulo `2^64`.
    #[must_use]
    #[inline]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_sub(rhs.0))
    }

    /// Wrapping fixed-point multiplication.
    ///
    /// The result is truncated toward zero at the scale of `Self` and wrapped modulo
    /// `2^64` if it is too large.
    #[must_use]
    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        Self::new(self.wide_mul(rhs) as u64)
    }

    /// Wrapping fixed-point division.
    ///
    /// The result is truncated toward zero at the scale of `Self` and wrapped modulo
    /// `2^64` if it is too large.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    #[must_use]
    #[inline]
    pub const fn wrapping_div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        Self::new(self.wide_div(rhs) as u64)
    }

    /// Overflowing fixed-point addition.
    ///
    /// Returns the wrapped result and whether overflow occurred.
    #[must_use]
    #[inline]
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (result, overflowed) = self.0.overflowing_add(rhs.0);
        (Self::new(result), overflowed)
    }

    /// Overflowing fixed-point subtraction.
    ///
    /// Returns the wrapped result and whether the result would be negative.
    #[must_use]
    #[inline]
    pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let (result, overflowed) = self.0.overflowing_sub(rhs.0);
        (Self::new(result), overflowed)
    }

    /// Overflowing fixed-point multiplication.
    ///
    /// The result is truncated toward zero at the scale of `Self`. Returns the result
    /// wrapped modulo `2^64` and whether overflow occurred.
    #[must_use]
    #[inline]
    pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let result = self.wide_mul(rhs);
        (Self::new(result as u64), result > u64::MAX as u128)
    }

    /// Overflowing fixed-point division.
    ///
    /// The result is truncated toward zero at the scale of `Self`. Returns the result
    /// wrapped modulo `2^64` and whether overflow occurred.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    #[must_use]
    #[inline]
    pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let result = self.wide_div(rhs);
        (Self::new(result as u64), result > u64::MAX as u128)
    }

    #[inline]
    const fn wide_mul(self, rhs: Self) -> u128 {
        let product = self.0 as u128 * rhs.0 as u128;
        product / S::SCALE_FACTOR as u128
    }

    #[inline]
    const fn wide_div(self, rhs: Self) -> u128 {
        let dividend = self.0 as u128 * S::SCALE_FACTOR as u128;
        dividend / rhs.0 as u128
    }
}

#[cfg(test)]
mod tests {
    mod checked_const {
        use crate::{DecimalU64, U2};

        type Decimal = DecimalU64<U2>;

        const ADD: Option<Decimal> = Decimal::ONE.checked_add(Decimal::TWO);
        const ADD_OVERFLOW: Option<Decimal> = Decimal::MAX.checked_add(Decimal::new(1));
        const SUB: Option<Decimal> = Decimal::TWO.checked_sub(Decimal::ONE);
        const SUB_UNDERFLOW: Option<Decimal> = Decimal::ZERO.checked_sub(Decimal::new(1));
        const MUL: Option<Decimal> = Decimal::TWO.checked_mul(Decimal::new(300));
        const MUL_OVERFLOW: Option<Decimal> = Decimal::MAX.checked_mul(Decimal::TWO);
        const DIV: Option<Decimal> = Decimal::new(600).checked_div(Decimal::TWO);
        const DIV_BY_ZERO: Option<Decimal> = Decimal::ONE.checked_div(Decimal::ZERO);
        const DIV_OVERFLOW: Option<Decimal> = Decimal::MAX.checked_div(Decimal::new(1));

        #[test]
        fn should_support_checked_arithmetic_in_const_contexts() {
            assert_eq!(ADD, Some(Decimal::new(300)));
            assert_eq!(ADD_OVERFLOW, None);
            assert_eq!(SUB, Some(Decimal::ONE));
            assert_eq!(SUB_UNDERFLOW, None);
            assert_eq!(MUL, Some(Decimal::new(600)));
            assert_eq!(MUL_OVERFLOW, None);
            assert_eq!(DIV, Some(Decimal::new(300)));
            assert_eq!(DIV_BY_ZERO, None);
            assert_eq!(DIV_OVERFLOW, None);
        }
    }

    mod operation_families {
        use crate::{DecimalU64, U2};

        type Decimal = DecimalU64<U2>;

        const SATURATING_ADD: Decimal = Decimal::MAX.saturating_add(Decimal::new(1));
        const SATURATING_SUB: Decimal = Decimal::ZERO.saturating_sub(Decimal::new(1));
        const SATURATING_MUL: Decimal = Decimal::MAX.saturating_mul(Decimal::TWO);
        const SATURATING_DIV: Decimal = Decimal::MAX.saturating_div(Decimal::new(1));

        const WRAPPING_ADD: Decimal = Decimal::MAX.wrapping_add(Decimal::new(1));
        const WRAPPING_SUB: Decimal = Decimal::ZERO.wrapping_sub(Decimal::new(1));
        const WRAPPING_MUL: Decimal = Decimal::MAX.wrapping_mul(Decimal::TWO);
        const WRAPPING_DIV: Decimal = Decimal::MAX.wrapping_div(Decimal::new(1));

        const OVERFLOWING_ADD: (Decimal, bool) = Decimal::MAX.overflowing_add(Decimal::new(1));
        const OVERFLOWING_SUB: (Decimal, bool) = Decimal::ZERO.overflowing_sub(Decimal::new(1));
        const OVERFLOWING_MUL: (Decimal, bool) = Decimal::MAX.overflowing_mul(Decimal::TWO);
        const OVERFLOWING_DIV: (Decimal, bool) = Decimal::MAX.overflowing_div(Decimal::new(1));

        #[test]
        fn should_saturate_at_numeric_bounds() {
            assert_eq!(SATURATING_ADD, Decimal::MAX);
            assert_eq!(SATURATING_SUB, Decimal::ZERO);
            assert_eq!(SATURATING_MUL, Decimal::MAX);
            assert_eq!(SATURATING_DIV, Decimal::MAX);
        }

        #[test]
        fn should_wrap_at_numeric_bounds() {
            assert_eq!(WRAPPING_ADD, Decimal::ZERO);
            assert_eq!(WRAPPING_SUB, Decimal::MAX);
            assert_eq!(WRAPPING_MUL, Decimal::new(u64::MAX - 1));
            assert_eq!(WRAPPING_DIV, Decimal::new(u64::MAX - 99));
        }

        #[test]
        fn should_report_overflow_and_return_wrapped_results() {
            assert_eq!(OVERFLOWING_ADD, (Decimal::ZERO, true));
            assert_eq!(OVERFLOWING_SUB, (Decimal::MAX, true));
            assert_eq!(OVERFLOWING_MUL, (Decimal::new(u64::MAX - 1), true));
            assert_eq!(OVERFLOWING_DIV, (Decimal::new(u64::MAX - 99), true));
        }

        #[test]
        fn should_not_report_overflow_for_representable_results() {
            assert_eq!(Decimal::ONE.overflowing_add(Decimal::TWO), (Decimal::THREE, false));
            assert_eq!(Decimal::TWO.overflowing_sub(Decimal::ONE), (Decimal::ONE, false));
            assert_eq!(Decimal::TWO.overflowing_mul(Decimal::THREE), (Decimal::SIX, false));
            assert_eq!(Decimal::SIX.overflowing_div(Decimal::TWO), (Decimal::THREE, false));
        }

        #[test]
        fn should_keep_operation_families_consistent() {
            let values = [
                Decimal::ZERO,
                Decimal::new(1),
                Decimal::new(99),
                Decimal::ONE,
                Decimal::new(u64::MAX / 2),
                Decimal::MAX,
            ];

            for lhs in values {
                for rhs in values {
                    let (add, add_overflowed) = lhs.overflowing_add(rhs);
                    assert_eq!(lhs.wrapping_add(rhs), add);
                    assert_eq!(lhs.checked_add(rhs), (!add_overflowed).then_some(add));
                    assert_eq!(lhs.saturating_add(rhs), if add_overflowed { Decimal::MAX } else { add });

                    let (sub, sub_overflowed) = lhs.overflowing_sub(rhs);
                    assert_eq!(lhs.wrapping_sub(rhs), sub);
                    assert_eq!(lhs.checked_sub(rhs), (!sub_overflowed).then_some(sub));
                    assert_eq!(
                        lhs.saturating_sub(rhs),
                        if sub_overflowed { Decimal::ZERO } else { sub }
                    );

                    let (mul, mul_overflowed) = lhs.overflowing_mul(rhs);
                    assert_eq!(lhs.wrapping_mul(rhs), mul);
                    assert_eq!(lhs.checked_mul(rhs), (!mul_overflowed).then_some(mul));
                    assert_eq!(lhs.saturating_mul(rhs), if mul_overflowed { Decimal::MAX } else { mul });

                    if rhs != Decimal::ZERO {
                        let (div, div_overflowed) = lhs.overflowing_div(rhs);
                        assert_eq!(lhs.wrapping_div(rhs), div);
                        assert_eq!(lhs.checked_div(rhs), (!div_overflowed).then_some(div));
                        assert_eq!(lhs.saturating_div(rhs), if div_overflowed { Decimal::MAX } else { div });
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn should_panic_when_saturating_divides_by_zero() {
            let _ = Decimal::ONE.saturating_div(Decimal::ZERO);
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn should_panic_when_wrapping_divides_by_zero() {
            let _ = Decimal::ONE.wrapping_div(Decimal::ZERO);
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn should_panic_when_overflowing_divides_by_zero() {
            let _ = Decimal::ONE.overflowing_div(Decimal::ZERO);
        }
    }

    mod operators {
        use crate::{DecimalU64, U2};

        type Decimal = DecimalU64<U2>;

        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn should_panic_when_addition_overflows() {
            let _ = Decimal::MAX + Decimal::new(1);
        }

        #[test]
        #[should_panic(expected = "attempt to subtract with overflow")]
        fn should_panic_when_subtraction_underflows() {
            let _ = Decimal::ZERO - Decimal::new(1);
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn should_panic_when_multiplication_overflows() {
            let _ = Decimal::MAX * Decimal::TWO;
        }

        #[test]
        #[should_panic(expected = "attempt to divide with overflow")]
        fn should_panic_when_division_overflows() {
            let _ = Decimal::MAX / Decimal::new(1);
        }
    }

    mod mul {
        use crate::{DecimalU64, U8};
        use rstest::rstest;

        #[rstest]
        #[case("0.2", "50000", "10000.00000000")]
        #[case("1", "1", "1.00000000")]
        #[case("0", "123.45", "0.00000000")]
        fn should_mul(#[case] a: &str, #[case] b: &str, #[case] expected: &str) {
            let dec_a = DecimalU64::<U8>::from_str(a).unwrap();
            let dec_b = DecimalU64::<U8>::from_str(b).unwrap();
            let result = dec_a.checked_mul(dec_b).unwrap();
            assert_eq!(expected, result.to_string());
            let result = dec_a * dec_b;
            assert_eq!(expected, result.to_string());
        }

        #[rstest]
        #[case("1000000000.00000000", "1000000000.00000000")]
        fn should_overflow(#[case] a: &str, #[case] b: &str) {
            let dec_a = DecimalU64::<U8>::from_str(a).unwrap();
            let dec_b = DecimalU64::<U8>::from_str(b).unwrap();
            assert!(dec_a.checked_mul(dec_b).is_none());
        }
    }

    mod add {
        use crate::{DecimalU64, U8};
        use rstest::rstest;

        #[rstest]
        #[case("0.2", "50000", "50000.20000000")]
        #[case("123.2", "50000", "50123.20000000")]
        #[case("0.2", "0", "0.20000000")]
        #[case("0", "0", "0.00000000")]
        #[case("123.45678901", "0.00000009", "123.45678910")]
        fn should_add_success(#[case] a: &str, #[case] b: &str, #[case] expected: &str) {
            let dec_a = DecimalU64::<U8>::from_str(a).unwrap();
            let dec_b = DecimalU64::<U8>::from_str(b).unwrap();
            let result = dec_a.checked_add(dec_b).unwrap();
            assert_eq!(expected, result.to_string());
            let result = dec_a + dec_b;
            assert_eq!(expected, result.to_string());
        }

        #[test]
        fn should_overflow() {
            // For U8, the maximum unscaled value is u64::MAX.
            // "184467440737.09551615" is the maximum in decimal notation.
            // Adding any positive amount should overflow.
            let dec_max = DecimalU64::<U8>::from_str("184467440737.09551615").unwrap();
            let dec_small = DecimalU64::<U8>::from_str("0.00000001").unwrap();
            assert!(dec_max.checked_add(dec_small).is_none());
        }
    }

    mod sub {
        use crate::{DecimalU64, U8};
        use rstest::rstest;

        #[rstest]
        #[case("50000", "0.2", "49999.80000000")]
        #[case("50000.02", "0.01", "50000.01000000")]
        #[case("123.45678910", "0.00000009", "123.45678901")]
        fn should_sub(#[case] a: &str, #[case] b: &str, #[case] expected: &str) {
            let dec_a = DecimalU64::<U8>::from_str(a).unwrap();
            let dec_b = DecimalU64::<U8>::from_str(b).unwrap();
            let result = dec_a.checked_sub(dec_b).unwrap();
            assert_eq!(expected, result.to_string());
            let result = dec_a - dec_b;
            assert_eq!(expected, result.to_string());
        }

        #[test]
        fn should_underflow() {
            let dec_zero = DecimalU64::<U8>::from_str("0.00000000").unwrap();
            let dec_sub = DecimalU64::<U8>::from_str("0.00000001").unwrap();
            assert!(dec_zero.checked_sub(dec_sub).is_none());
        }
    }

    mod div {
        use crate::{DecimalU64, U8};
        use rstest::rstest;

        #[rstest]
        #[case("50000", "0.2", "250000.00000000")]
        #[case("123.45678901", "2", "61.72839450")]
        #[case("0", "123.45678901", "0.00000000")]
        #[case("1", "3", "0.33333333")]
        #[case("0.129", "0.01", "12.90000000")]
        fn should_div(#[case] a: &str, #[case] b: &str, #[case] expected: &str) {
            let dec_a = DecimalU64::<U8>::from_str(a).unwrap();
            let dec_b = DecimalU64::<U8>::from_str(b).unwrap();
            let result = dec_a.checked_div(dec_b).unwrap();
            assert_eq!(expected, result.to_string());
            let result = dec_a / dec_b;
            assert_eq!(expected, result.to_string());
        }

        #[test]
        fn should_not_checked_div_by_zero() {
            let dec_a = DecimalU64::<U8>::from_str("123.45678901").unwrap();
            let dec_zero = DecimalU64::<U8>::ZERO;
            assert!(dec_a.checked_div(dec_zero).is_none());
        }

        #[test]
        #[should_panic = "attempt to divide by zero"]
        fn should_panic_if_div_by_zero() {
            let dec_a = DecimalU64::<U8>::from_str("123.45678901").unwrap();
            let dec_zero = DecimalU64::<U8>::ZERO;
            let _ = dec_a / dec_zero;
        }

        #[test]
        fn should_overflow() {
            // Dividing a very large number by a very small number should overflow.
            let dec_max = DecimalU64::<U8>::from_str("184467440737.09551615").unwrap();
            let dec_small = DecimalU64::<U8>::from_str("0.00000001").unwrap();
            assert!(dec_max.checked_div(dec_small).is_none());
        }
    }

    mod assign {
        use crate::{DecimalU64, U8};

        #[test]
        fn should_apply_owned_and_borrowed_assignments() {
            let mut value = DecimalU64::<U8>::from_str("6").unwrap();
            let two = DecimalU64::<U8>::TWO;

            value += two;
            value -= &two;
            value *= two;
            value /= &two;

            assert_eq!("6.00000000", value.to_string());
        }
    }

    mod iterator_arithmetic {
        use crate::{DecimalU64, U8};

        #[test]
        fn should_sum_values() {
            let values: Vec<DecimalU64<U8>> = vec![];
            let sum = values.iter().sum::<DecimalU64<U8>>();
            assert_eq!(sum, DecimalU64::ZERO);

            let values: Vec<DecimalU64<U8>> = vec![DecimalU64::ONE, DecimalU64::TWO];
            let sum = values.iter().sum::<DecimalU64<U8>>();
            assert_eq!(sum, DecimalU64::THREE);
        }

        #[test]
        fn should_multiply_values() {
            let values: Vec<DecimalU64<U8>> = vec![];
            let product = values.iter().product::<DecimalU64<U8>>();
            assert_eq!(product, DecimalU64::ONE);

            let values = vec![DecimalU64::<U8>::TWO, DecimalU64::THREE];
            let product = values.into_iter().product::<DecimalU64<U8>>();
            assert_eq!(product, DecimalU64::SIX);
        }
    }
}
