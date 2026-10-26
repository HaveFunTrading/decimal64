use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{ToPrimitive, Zero};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ArithmeticOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct U64ArithmeticOutcome {
    pub exact: BigInt,
    pub checked: Option<u64>,
    pub wrapping: u64,
    pub overflowing: bool,
    pub saturating: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct I64ArithmeticOutcome {
    pub exact: BigInt,
    pub checked: Option<i64>,
    pub wrapping: i64,
    pub overflowing: bool,
    pub saturating: i64,
}

pub(crate) fn trunc_div_rem(numerator: &BigInt, denominator: &BigInt) -> (BigInt, BigInt) {
    assert!(!denominator.is_zero());

    let magnitude = numerator.magnitude() / denominator.magnitude();
    let sign = if magnitude.is_zero() {
        Sign::NoSign
    } else if numerator.sign() == denominator.sign() {
        Sign::Plus
    } else {
        Sign::Minus
    };
    let quotient = BigInt::from_biguint(sign, magnitude);
    let remainder = numerator - &quotient * denominator;
    (quotient, remainder)
}

fn exact_binary_result(lhs: BigInt, rhs: BigInt, factor: u64, operation: ArithmeticOperation) -> Option<BigInt> {
    match operation {
        ArithmeticOperation::Add => Some(lhs + rhs),
        ArithmeticOperation::Sub => Some(lhs - rhs),
        ArithmeticOperation::Mul => {
            let numerator = lhs * rhs;
            Some(trunc_div_rem(&numerator, &BigInt::from(factor)).0)
        }
        ArithmeticOperation::Div => {
            if rhs.is_zero() {
                None
            } else {
                let numerator = lhs * BigInt::from(factor);
                Some(trunc_div_rem(&numerator, &rhs).0)
            }
        }
    }
}

fn wrapping_u64(value: &BigInt) -> u64 {
    let modulus = BigInt::from(1_u8) << 64_usize;
    let mut wrapped = value % &modulus;
    if wrapped.sign() == Sign::Minus {
        wrapped += modulus;
    }
    wrapped.to_u64().unwrap()
}

pub(crate) fn unsigned_arithmetic_outcome(
    lhs: u64,
    rhs: u64,
    factor: u64,
    operation: ArithmeticOperation,
) -> Option<U64ArithmeticOutcome> {
    let exact = exact_binary_result(lhs.into(), rhs.into(), factor, operation)?;
    let checked = exact.to_u64();
    let overflowing = checked.is_none();
    let saturating = if exact.sign() == Sign::Minus {
        0
    } else {
        checked.unwrap_or(u64::MAX)
    };

    Some(U64ArithmeticOutcome {
        wrapping: wrapping_u64(&exact),
        exact,
        checked,
        overflowing,
        saturating,
    })
}

pub(crate) fn signed_arithmetic_outcome(
    lhs: i64,
    rhs: i64,
    factor: u64,
    operation: ArithmeticOperation,
) -> Option<I64ArithmeticOutcome> {
    let exact = exact_binary_result(lhs.into(), rhs.into(), factor, operation)?;
    let checked = exact.to_i64();
    let overflowing = checked.is_none();
    let saturating = if exact < BigInt::from(i64::MIN) {
        i64::MIN
    } else if exact > BigInt::from(i64::MAX) {
        i64::MAX
    } else {
        checked.unwrap()
    };

    Some(I64ArithmeticOutcome {
        wrapping: wrapping_u64(&exact) as i64,
        exact,
        checked,
        overflowing,
        saturating,
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum FloatError {
    Negative,
    Infinite,
    NotANumber,
    Overflow,
}

pub(crate) fn format_fixed(raw: u64, scale: usize, factor: u64) -> String {
    if scale == 0 {
        raw.to_string()
    } else {
        format!("{}.{:0scale$}", raw / factor, raw % factor)
    }
}

pub(crate) fn format_trimmed(raw: u64, scale: usize, factor: u64) -> String {
    let mut formatted = format_fixed(raw, scale, factor);
    if scale != 0 {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }
    formatted
}

pub(crate) fn format_signed_fixed(raw: i64, scale: usize, factor: u64) -> String {
    let magnitude = raw.unsigned_abs();
    let sign = if raw < 0 { "-" } else { "" };
    format!("{sign}{}", format_fixed(magnitude, scale, factor))
}

pub(crate) fn format_signed_trimmed(raw: i64, scale: usize, factor: u64) -> String {
    let magnitude = raw.unsigned_abs();
    let sign = if raw < 0 { "-" } else { "" };
    format!("{sign}{}", format_trimmed(magnitude, scale, factor))
}

pub(crate) fn from_f64(value: f64, factor: u64) -> Result<u64, FloatError> {
    const EXPONENT_MASK: u64 = 0x7ff;
    const FRACTION_BITS: u32 = 52;
    const FRACTION_MASK: u64 = (1_u64 << FRACTION_BITS) - 1;
    const EXPONENT_BIAS: i32 = 1023;

    let bits = value.to_bits();
    let sign = bits >> 63;
    let exponent = (bits >> FRACTION_BITS) & EXPONENT_MASK;
    let fraction = bits & FRACTION_MASK;

    if exponent == EXPONENT_MASK {
        return if fraction == 0 {
            Err(FloatError::Infinite)
        } else {
            Err(FloatError::NotANumber)
        };
    }
    if sign != 0 && (exponent != 0 || fraction != 0) {
        return Err(FloatError::Negative);
    }
    if exponent == 0 && fraction == 0 {
        return Ok(0);
    }

    let (mantissa, exponent2) = if exponent == 0 {
        (fraction, 1 - EXPONENT_BIAS - FRACTION_BITS as i32)
    } else {
        (
            (1_u64 << FRACTION_BITS) | fraction,
            exponent as i32 - EXPONENT_BIAS - FRACTION_BITS as i32,
        )
    };

    let mut numerator = BigUint::from(mantissa) * BigUint::from(factor);
    if exponent2 >= 0 {
        numerator <<= exponent2 as usize;
    } else {
        let denominator = BigUint::from(1_u8) << (-exponent2) as usize;
        let remainder = &numerator % &denominator;
        numerator /= &denominator;
        if (remainder << 1_usize) >= denominator {
            numerator += 1_u8;
        }
    }

    numerator.to_u64().ok_or(FloatError::Overflow)
}

pub(crate) fn signed_from_f64(value: f64, factor: u64) -> Result<i64, FloatError> {
    let negative = value.is_sign_negative();
    let magnitude = from_f64(value.abs(), factor)?;
    if negative {
        if magnitude > 1_u64 << 63 {
            Err(FloatError::Overflow)
        } else if magnitude == 1_u64 << 63 {
            Ok(i64::MIN)
        } else {
            Ok(-(magnitude as i64))
        }
    } else if magnitude > i64::MAX as u64 {
        Err(FloatError::Overflow)
    } else {
        Ok(magnitude as i64)
    }
}
