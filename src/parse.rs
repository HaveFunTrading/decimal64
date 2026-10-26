//! Exact parsing for signed and unsigned decimals.

use crate::scale::SCALE_FACTORS;
use crate::{DecimalI64, DecimalU64, Error, InvalidInputKind, ScaleMetrics};

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Parses an exactly representable decimal from an ASCII byte slice.
    ///
    /// Fractional digits beyond the target scale are accepted only when they
    /// are zero. This method never rounds.
    ///
    /// # Example
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::from_slice(b"12.34").unwrap();
    /// assert_eq!("12.34", value.to_string());
    /// ```
    #[inline]
    pub const fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidInput(InvalidInputKind::Empty));
        }

        let mut index: usize = 0;

        match bytes[0] {
            b'-' => return Err(Error::InvalidInput(InvalidInputKind::NegativeNumber)),
            b'+' => index = 1,
            _ => {}
        }

        match parse_magnitude::<S>(bytes, index) {
            Ok(unscaled) => Ok(Self::new(unscaled)),
            Err(error) => Err(error),
        }
    }
}

#[inline(always)]
const fn parse_magnitude<S: ScaleMetrics>(bytes: &[u8], mut index: usize) -> Result<u64, Error> {
    let input_len = bytes.len() - index;
    let scale = S::SCALE as usize;

    // Fixed-scale wire formats are common in market data. When the point
    // is exactly where the target scale requires it, parse both fields
    // independently and perform only two final overflow checks.
    if scale != 0 && input_len > scale {
        let decimal_point = bytes.len() - scale - 1;
        if bytes[decimal_point] == b'.' && decimal_point - index <= 19 {
            return parse_exact_scale_slice::<S>(bytes, index, decimal_point);
        }
    }

    // Any decimal with at most 19 effective digits fits in a `u64`. The
    // input length is a conservative upper bound for its digit count, so
    // these inputs need no overflow checks at all. `ScaleMetrics` is
    // sealed to U0–U8, making this subtraction safe.
    if input_len <= 19 - scale {
        return parse_short_slice::<S, false>(bytes, index);
    }

    // Up to 19 input digits also fit in the accumulator. Only the final
    // scale adjustment can overflow, reducing the hot loop to unchecked
    // arithmetic and a single checked multiplication.
    if input_len <= 19 {
        return parse_short_slice::<S, true>(bytes, index);
    }

    let mut unscaled: u64 = 0;
    let mut fractional_digits: usize = 0;
    let mut seen_decimal_point = false;
    let mut seen_digit = false;

    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'0'..=b'9' => {
                seen_digit = true;
                let digit = (byte - b'0') as u64;

                if seen_decimal_point && fractional_digits == S::SCALE as usize {
                    if digit != 0 {
                        return Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss));
                    }
                    index += 1;
                    continue;
                }

                let next = match unscaled.checked_mul(10) {
                    Some(value) => value,
                    None => return Err(Error::Overflow),
                };
                unscaled = match next.checked_add(digit) {
                    Some(value) => value,
                    None => return Err(Error::Overflow),
                };

                if seen_decimal_point {
                    fractional_digits += 1;
                }
            }
            b'.' => {
                if seen_decimal_point {
                    return Err(Error::InvalidInput(InvalidInputKind::MultipleDecimalPoints));
                }
                seen_decimal_point = true;
            }
            other => return Err(Error::InvalidInput(InvalidInputKind::InvalidByte(other))),
        }

        index += 1;
    }

    if !seen_digit {
        return Err(Error::InvalidInput(InvalidInputKind::MissingDigits));
    }

    let remaining_scale = S::SCALE as usize - fractional_digits;
    let factor = SCALE_FACTORS[remaining_scale];
    let unscaled = match unscaled.checked_mul(factor) {
        Some(unscaled) => unscaled,
        None => return Err(Error::Overflow),
    };

    Ok(unscaled)
}

#[inline(always)]
const fn parse_exact_scale_slice<S: ScaleMetrics>(
    bytes: &[u8],
    index: usize,
    decimal_point: usize,
) -> Result<u64, Error> {
    let integer_digits = decimal_point - index;
    let integer = if integer_digits <= 8 {
        parse_up_to_eight_digits(bytes, index, integer_digits)
    } else {
        parse_up_to_nineteen_digits(bytes, index, integer_digits)
    };
    let integer = match integer {
        Some(integer) => integer,
        None => return Err(invalid_digits(bytes, index, decimal_point)),
    };

    let fraction_index = decimal_point + 1;
    let fraction = match parse_up_to_eight_digits(bytes, fraction_index, S::SCALE as usize) {
        Some(fraction) => fraction,
        None => return Err(invalid_digits(bytes, fraction_index, bytes.len())),
    };

    let unscaled = match integer.checked_mul(S::SCALE_FACTOR) {
        Some(unscaled) => unscaled,
        None => return Err(Error::Overflow),
    };
    let unscaled = match unscaled.checked_add(fraction) {
        Some(unscaled) => unscaled,
        None => return Err(Error::Overflow),
    };
    Ok(unscaled)
}

#[inline(always)]
const fn parse_short_slice<S: ScaleMetrics, const CHECK_SCALE_OVERFLOW: bool>(
    bytes: &[u8],
    mut index: usize,
) -> Result<u64, Error> {
    if bytes.len() - index == 8
        && let Some(integer) = parse_eight_digits(bytes, index)
    {
        let unscaled = integer as u64 * S::SCALE_FACTOR;
        return Ok(unscaled);
    }

    let mut unscaled: u64 = 0;
    let mut seen_digits = 0usize;
    let mut seen_decimal_point = false;

    while index < bytes.len() {
        let byte = bytes[index];
        let digit = byte.wrapping_sub(b'0');
        if digit <= 9 {
            unscaled = unscaled * 10 + digit as u64;
            seen_digits += 1;
            index += 1;
        } else if byte == b'.' {
            seen_decimal_point = true;
            index += 1;
            break;
        } else {
            return Err(Error::InvalidInput(InvalidInputKind::InvalidByte(byte)));
        }
    }

    let mut fractional_digits = 0usize;
    if seen_decimal_point {
        let remaining = bytes.len() - index;
        if remaining <= S::SCALE as usize
            && let Some(fraction) = parse_up_to_eight_digits(bytes, index, remaining)
        {
            unscaled = unscaled * SCALE_FACTORS[remaining] + fraction;
            fractional_digits = remaining;
            seen_digits += remaining;
            index += remaining;
        }

        while index < bytes.len() {
            let byte = bytes[index];
            let digit = byte.wrapping_sub(b'0');
            if digit > 9 {
                if byte == b'.' {
                    return Err(Error::InvalidInput(InvalidInputKind::MultipleDecimalPoints));
                }
                return Err(Error::InvalidInput(InvalidInputKind::InvalidByte(byte)));
            }

            seen_digits += 1;
            if fractional_digits < S::SCALE as usize {
                unscaled = unscaled * 10 + digit as u64;
                fractional_digits += 1;
            } else if digit != 0 {
                return Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss));
            }
            index += 1;
        }
    }

    if seen_digits == 0 {
        return Err(Error::InvalidInput(InvalidInputKind::MissingDigits));
    }

    let remaining_scale = S::SCALE as usize - fractional_digits;
    let factor = SCALE_FACTORS[remaining_scale];
    let unscaled = if CHECK_SCALE_OVERFLOW {
        match unscaled.checked_mul(factor) {
            Some(unscaled) => unscaled,
            None => return Err(Error::Overflow),
        }
    } else {
        unscaled * factor
    };
    Ok(unscaled)
}

impl<S: ScaleMetrics> DecimalI64<S> {
    /// Parses an exactly representable decimal from an ASCII byte slice.
    ///
    /// An optional leading `+` or `-` is accepted. Fractional digits beyond
    /// the target scale are accepted only when they are zero. This method
    /// never rounds, and negative zero is normalized to zero.
    #[inline]
    pub const fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidInput(InvalidInputKind::Empty));
        }

        let (negative, index) = match bytes[0] {
            b'-' => (true, 1),
            b'+' => (false, 1),
            _ => (false, 0),
        };
        let magnitude = match parse_magnitude::<S>(bytes, index) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        if negative {
            if magnitude > (1u64 << 63) {
                return Err(Error::Overflow);
            }
            if magnitude == (1u64 << 63) {
                Ok(Self::MIN)
            } else {
                Ok(Self::new(-(magnitude as i64)))
            }
        } else if magnitude > i64::MAX as u64 {
            Err(Error::Overflow)
        } else {
            Ok(Self::new(magnitude as i64))
        }
    }
}

impl_decimal_string_parsing!(DecimalU64);
impl_decimal_string_parsing!(DecimalI64);

/// Parses eight validated ASCII digits using packed integer arithmetic.
#[inline(always)]
pub(crate) const fn parse_eight_digits(bytes: &[u8], index: usize) -> Option<u32> {
    let value = u64::from_le_bytes([
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
        bytes[index + 4],
        bytes[index + 5],
        bytes[index + 6],
        bytes[index + 7],
    ]);

    let upper = value.wrapping_add(0x4646_4646_4646_4646);
    let lower = value.wrapping_sub(0x3030_3030_3030_3030);
    if ((upper | lower) & 0x8080_8080_8080_8080) != 0 {
        return None;
    }

    let value = lower;
    let value = (value.wrapping_mul(10).wrapping_add(value >> 8)) & 0x00ff_00ff_00ff_00ff;
    let value = (value.wrapping_mul(100).wrapping_add(value >> 16)) & 0x0000_ffff_0000_ffff;
    let value = value.wrapping_mul(10_000).wrapping_add(value >> 32);
    Some(value as u32)
}

#[inline(always)]
const fn parse_up_to_eight_digits(bytes: &[u8], index: usize, len: usize) -> Option<u64> {
    match len {
        0 => Some(0),
        1 => parse_digits::<1>(bytes, index),
        2 => parse_digits::<2>(bytes, index),
        3 => parse_digits::<3>(bytes, index),
        4 => parse_digits::<4>(bytes, index),
        5 => parse_digits::<5>(bytes, index),
        6 => parse_digits::<6>(bytes, index),
        7 => parse_digits::<7>(bytes, index),
        8 => match parse_eight_digits(bytes, index) {
            Some(value) => Some(value as u64),
            None => None,
        },
        _ => None,
    }
}

#[inline(never)]
const fn parse_up_to_nineteen_digits(bytes: &[u8], index: usize, len: usize) -> Option<u64> {
    if len > 19 {
        return None;
    }
    if len <= 8 {
        return parse_up_to_eight_digits(bytes, index, len);
    }

    if len <= 16 {
        let high_len = len - 8;
        let high = match parse_up_to_eight_digits(bytes, index, high_len) {
            Some(value) => value,
            None => return None,
        };
        let low = match parse_eight_digits(bytes, index + high_len) {
            Some(value) => value as u64,
            None => return None,
        };
        return Some(high * 100_000_000 + low);
    }

    let high_len = len - 16;
    let high = match parse_up_to_eight_digits(bytes, index, high_len) {
        Some(value) => value,
        None => return None,
    };
    let middle = match parse_eight_digits(bytes, index + high_len) {
        Some(value) => value as u64,
        None => return None,
    };
    let low = match parse_eight_digits(bytes, index + high_len + 8) {
        Some(value) => value as u64,
        None => return None,
    };
    Some(high * 10_000_000_000_000_000 + middle * 100_000_000 + low)
}

#[inline(always)]
const fn parse_digits<const LEN: usize>(bytes: &[u8], index: usize) -> Option<u64> {
    let mut value = 0u64;
    let mut offset = 0usize;
    while offset < LEN {
        let digit = bytes[index + offset].wrapping_sub(b'0');
        if digit > 9 {
            return None;
        }
        value = value * 10 + digit as u64;
        offset += 1;
    }
    Some(value)
}

#[cold]
#[inline(never)]
const fn invalid_digits(bytes: &[u8], mut index: usize, end: usize) -> Error {
    while index < end {
        let byte = bytes[index];
        if byte.wrapping_sub(b'0') > 9 {
            return if byte == b'.' {
                Error::InvalidInput(InvalidInputKind::MultipleDecimalPoints)
            } else {
                Error::InvalidInput(InvalidInputKind::InvalidByte(byte))
            };
        }
        index += 1;
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::parse_eight_digits;

    #[test]
    fn packed_parser_accepts_exactly_eight_ascii_digits() {
        for expected in [0, 1, 12_345_678, 50_607_080, 99_999_999] {
            let digits = format!("{expected:08}");
            assert_eq!(parse_eight_digits(digits.as_bytes(), 0), Some(expected));
        }
    }

    #[test]
    fn packed_parser_rejects_every_non_digit_byte() {
        for index in 0..8 {
            for invalid in 0..=u8::MAX {
                if invalid.is_ascii_digit() {
                    continue;
                }
                let mut digits = *b"12345678";
                digits[index] = invalid;
                assert_eq!(parse_eight_digits(&digits, 0), None);
            }
        }
    }
}
