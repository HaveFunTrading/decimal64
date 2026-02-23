use crate::error::{Error, InvalidInputKind};
use crate::{DecimalU64, ScaleMetrics};

const INTERNAL_SCALE: u32 = 18;
const INTERNAL_FACTOR: u128 = 1_000_000_000_000_000_000;
const LN2_INTERNAL: u128 = 693_147_180_559_945_309;
const EXP_TERMS: u32 = 18;
const LN_TERMS: u32 = 12;

const POW10_U128: [u128; 19] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

const fn scale_to_internal(unscaled: u64, scale: u8) -> Result<u128, Error> {
    if scale as u32 > INTERNAL_SCALE {
        return Err(Error::Overflow);
    }

    let factor = POW10_U128[(INTERNAL_SCALE - scale as u32) as usize];
    match (unscaled as u128).checked_mul(factor) {
        Some(value) => Ok(value),
        None => Err(Error::Overflow),
    }
}

const fn scale_from_internal(value: u128, scale: u8) -> Result<u64, Error> {
    if scale as u32 > INTERNAL_SCALE {
        return Err(Error::Overflow);
    }

    let factor = POW10_U128[(INTERNAL_SCALE - scale as u32) as usize];
    let mut unscaled = value / factor;
    let remainder = value % factor;
    if remainder * 2 >= factor {
        unscaled += 1;
    }
    if unscaled > u64::MAX as u128 {
        return Err(Error::Overflow);
    }

    Ok(unscaled as u64)
}

const fn mul_scaled(a: u128, b: u128) -> Result<u128, Error> {
    match a.checked_mul(b) {
        Some(product) => Ok(product / INTERNAL_FACTOR),
        None => Err(Error::Overflow),
    }
}

const fn div_scaled(a: u128, b: u128) -> Result<u128, Error> {
    if b == 0 {
        return Err(Error::Overflow);
    }
    match a.checked_mul(INTERNAL_FACTOR) {
        Some(numerator) => Ok(numerator / b),
        None => Err(Error::Overflow),
    }
}

const fn exp_internal(x: u128) -> Result<u128, Error> {
    let k = x / LN2_INTERNAL;
    if k >= 128 {
        return Err(Error::Overflow);
    }
    let k = k as u32;
    let r = x - (k as u128) * LN2_INTERNAL;

    let mut term = INTERNAL_FACTOR;
    let mut sum = INTERNAL_FACTOR;
    let mut n: u32 = 1;
    while n <= EXP_TERMS {
        term = match mul_scaled(term, r) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        term /= n as u128;
        sum = match sum.checked_add(term) {
            Some(value) => value,
            None => return Err(Error::Overflow),
        };
        n += 1;
    }

    if k >= 128 {
        return Err(Error::Overflow);
    }
    if sum > (u128::MAX >> k) {
        return Err(Error::Overflow);
    }

    Ok(sum << k)
}

const fn ln_internal(x: u128) -> Result<u128, Error> {
    if x < INTERNAL_FACTOR {
        return Err(Error::InvalidInput(InvalidInputKind::LessThanOne));
    }

    let mut value = x;
    let mut k: u32 = 0;
    while value >= INTERNAL_FACTOR * 2 {
        value /= 2;
        k += 1;
    }

    let numerator = value - INTERNAL_FACTOR;
    let denominator = value + INTERNAL_FACTOR;
    let z = match div_scaled(numerator, denominator) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let z2 = match mul_scaled(z, z) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };

    let mut term = z;
    let mut sum = z;
    let mut n: u32 = 1;
    while n < LN_TERMS {
        term = match mul_scaled(term, z2) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let denom = (2 * n + 1) as u128;
        let addend = term / denom;
        sum = match sum.checked_add(addend) {
            Some(value) => value,
            None => return Err(Error::Overflow),
        };
        n += 1;
    }

    let mut result = match sum.checked_mul(2) {
        Some(value) => value,
        None => return Err(Error::Overflow),
    };
    let k_ln2 = match (k as u128).checked_mul(LN2_INTERNAL) {
        Some(value) => value,
        None => return Err(Error::Overflow),
    };
    result = match result.checked_add(k_ln2) {
        Some(value) => value,
        None => return Err(Error::Overflow),
    };
    Ok(result)
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Computes the natural logarithm, returning an error for values less than one.
    pub const fn ln(self) -> Result<Self, Error> {
        let value = match scale_to_internal(self.0, S::SCALE) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let result = match ln_internal(value) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let unscaled = match scale_from_internal(result, S::SCALE) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        Ok(DecimalU64::new(unscaled))
    }

    /// Computes the natural exponential, returning an error on overflow.
    pub const fn exp(self) -> Result<Self, Error> {
        let value = match scale_to_internal(self.0, S::SCALE) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let result = match exp_internal(value) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let unscaled = match scale_from_internal(result, S::SCALE) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        Ok(DecimalU64::new(unscaled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DecimalU64, U0, U6};
    use rstest_macros::rstest;
    use std::str::FromStr;

    fn assert_close(actual: DecimalU64<U6>, expected: &str, tolerance: u64) {
        let expected = DecimalU64::<U6>::from_str(expected).unwrap();
        let diff = actual.0.abs_diff(expected.0);
        assert!(diff <= tolerance);
    }

    #[test]
    fn should_exp_zero() {
        let out = DecimalU64::<U6>::ZERO.exp().unwrap();
        assert_eq!(DecimalU64::<U6>::ONE, out);
    }

    #[test]
    fn should_ln_one() {
        let out = DecimalU64::<U6>::ONE.ln().unwrap();
        assert_eq!(DecimalU64::<U6>::ZERO, out);
    }

    #[test]
    fn should_exp_one_close() {
        let out = DecimalU64::<U6>::ONE.exp().unwrap();
        assert_close(out, "2.718282", 3);
    }

    #[test]
    fn should_ln_two_close() {
        let value = DecimalU64::<U6>::from_str("2").unwrap();
        let out = value.ln().unwrap();
        assert_close(out, "0.693147", 3);
    }

    #[test]
    fn should_error_ln_less_than_one() {
        let value = DecimalU64::<U6>::from_str("0.5").unwrap();
        let err = value.ln();
        assert!(matches!(err, Err(Error::InvalidInput(InvalidInputKind::LessThanOne))));
    }

    #[test]
    fn should_error_exp_overflow() {
        let value = DecimalU64::<U0>::from_str("1000").unwrap();
        let err = value.exp();
        assert!(matches!(err, Err(Error::Overflow)));
    }

    fn assert_close_f64(actual: f64, expected: f64, tolerance: f64) {
        let diff = if actual >= expected {
            actual - expected
        } else {
            expected - actual
        };
        assert!(diff <= tolerance);
    }

    #[rstest]
    #[case("1")]
    #[case("1.5")]
    #[case("2")]
    #[case("3.141593")]
    #[case("10")]
    fn should_ln_match_f64(#[case] input: &str) {
        let value = DecimalU64::<U6>::from_str(input).unwrap();
        let actual = value.ln().unwrap().to_f64();
        let expected = f64::from_str(input).unwrap().ln();
        assert_close_f64(actual, expected, 5e-6);
    }

    #[rstest]
    #[case("0")]
    #[case("0.1")]
    #[case("0.5")]
    #[case("1")]
    #[case("2")]
    #[case("5")]
    fn should_exp_match_f64(#[case] input: &str) {
        let value = DecimalU64::<U6>::from_str(input).unwrap();
        let actual = value.exp().unwrap().to_f64();
        let expected = f64::from_str(input).unwrap().exp();
        assert_close_f64(actual, expected, 5e-6);
    }
}
