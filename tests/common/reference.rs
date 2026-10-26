use num_bigint::BigUint;
use num_traits::ToPrimitive;

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
