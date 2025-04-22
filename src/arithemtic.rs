use crate::{DecimalU64, ScaleMetrics};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

impl<S: ScaleMetrics> Mul for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let product = self.unscaled as u128 * rhs.unscaled as u128;
        let scale_factor = S::SCALE_FACTOR as u128;
        Self::from_raw((product / scale_factor) as u64)
    }
}

impl<S: ScaleMetrics> Add for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.unscaled + rhs.unscaled;
        Self::from_raw(sum)
    }
}

impl<S: ScaleMetrics> Sub for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let diff = self.unscaled - rhs.unscaled;
        Self::from_raw(diff)
    }
}

impl<S: ScaleMetrics> Div for DecimalU64<S> {
    type Output = DecimalU64<S>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.unscaled == 0 {
            panic!("Division by zero");
        }
        let dividend = self.unscaled as u128 * S::SCALE_FACTOR as u128;
        let quotient = dividend / (rhs.unscaled as u128);
        Self::from_raw(quotient as u64)
    }
}

impl<S: ScaleMetrics> AddAssign for DecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.unscaled += rhs.unscaled;
    }
}

impl<S: ScaleMetrics> SubAssign for DecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.unscaled -= rhs.unscaled;
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Multiply two decimals with the same scale.
    /// This performs the multiplication in u128 and then scales the result down by dividing by `S::SCALE_FACTOR`.
    /// It returns an error if an overflow occurs.
    #[inline]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        // multiply in u128 to avoid overflow in the intermediate product
        let product = (self.unscaled as u128).checked_mul(other.unscaled as u128)?;

        // divide by the scale factor to maintain the same scale
        let scale_factor = S::SCALE_FACTOR as u128;
        let result = product / scale_factor;

        // ensure the result fits back into a u64
        if result > u64::MAX as u128 {
            None
        } else {
            Some(Self::from_raw(result as u64))
        }
    }

    /// Add two decimals with the same scale.
    #[inline]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        let sum = self.unscaled.checked_add(other.unscaled)?;
        Some(Self::from_raw(sum))
    }

    /// Subtract one decimal from another. Returns an error if underflow occurs.
    #[inline]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let diff = self.unscaled.checked_sub(other.unscaled)?;
        Some(Self::from_raw(diff))
    }

    /// Divide one decimal by another using 128-bit arithmetic for the intermediate computation.
    /// The result is computed as (self.unscaled * SCALE_FACTOR) / other.unscaled.
    #[inline]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.unscaled == 0 {
            return None;
        }
        let dividend = (self.unscaled as u128).checked_mul(S::SCALE_FACTOR as u128)?;
        let quotient = dividend / (other.unscaled as u128);
        if quotient > u64::MAX as u128 {
            None
        } else {
            Some(Self::from_raw(quotient as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    mod mul {
        use crate::{DecimalU64, U8};
        use rstest_macros::rstest;
        use std::str::FromStr;

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
        use rstest_macros::rstest;
        use std::str::FromStr;

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
        use rstest_macros::rstest;
        use std::str::FromStr;

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
        use rstest_macros::rstest;
        use std::str::FromStr;

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
        #[should_panic = "Division by zero"]
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
        use std::str::FromStr;

        #[test]
        fn should_add_and_sub_assign() {
            let mut one = DecimalU64::<U8>::from_str("100").unwrap();
            let two = DecimalU64::<U8>::from_str("200").unwrap();
            one += two;
            assert_eq!("300.00000000", one.to_string());
            one -= two;
            assert_eq!("100.00000000", one.to_string());
        }
    }
}
