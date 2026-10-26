//! Allocation-free formatting for signed and unsigned decimals.

use crate::{BufferTooSmall, DecimalI64, DecimalU64, ScaleMetrics};

const DIGIT_PAIRS: [u8; 200] = make_digit_pairs();

const fn make_digit_pairs() -> [u8; 200] {
    let mut pairs = [0u8; 200];
    let mut value = 0usize;
    while value < 100 {
        pairs[value * 2] = b'0' + (value / 10) as u8;
        pairs[value * 2 + 1] = b'0' + (value % 10) as u8;
        value += 1;
    }
    pairs
}

#[inline(always)]
fn write_digit_pair(buffer: &mut [u8], index: usize, value: usize) {
    let pair = value * 2;
    // SAFETY: callers guarantee value < 100 and sufficient buffer capacity.
    unsafe {
        *buffer.get_unchecked_mut(index) = *DIGIT_PAIRS.get_unchecked(pair);
        *buffer.get_unchecked_mut(index + 1) = *DIGIT_PAIRS.get_unchecked(pair + 1);
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Splits the raw value into integer and fractional components.
    #[inline]
    pub const fn split(self) -> (u64, u64) {
        (self.0 / S::SCALE_FACTOR, self.0 % S::SCALE_FACTOR)
    }

    #[inline]
    fn try_write_impl(&self, buffer: &mut [u8], trimmed: bool) -> Result<usize, BufferTooSmall> {
        let (integer, mut fractional) = self.split();
        let mut fractional_digits = S::SCALE as usize;
        if trimmed {
            while fractional_digits != 0 && fractional.is_multiple_of(10) {
                fractional /= 10;
                fractional_digits -= 1;
            }
        }
        try_write_parts(buffer, integer, fractional, fractional_digits)
    }
}

impl<S: ScaleMetrics> DecimalI64<S> {
    /// Splits the raw value into integer and fractional components.
    ///
    /// Both components retain the sign and use truncation toward zero. For
    /// example, `-12.34` splits into `(-12, -34)` at scale two.
    #[inline]
    pub const fn split(self) -> (i64, i64) {
        let factor = S::SCALE_FACTOR as i64;
        (self.0 / factor, self.0 % factor)
    }

    #[inline]
    fn try_write_impl(&self, buffer: &mut [u8], trimmed: bool) -> Result<usize, BufferTooSmall> {
        let magnitude = self.0.unsigned_abs();
        let integer = magnitude / S::SCALE_FACTOR;
        let mut fractional = magnitude % S::SCALE_FACTOR;
        let mut fractional_digits = S::SCALE as usize;
        if trimmed {
            while fractional_digits != 0 && fractional.is_multiple_of(10) {
                fractional /= 10;
                fractional_digits -= 1;
            }
        }

        if self.0 >= 0 {
            return try_write_parts(buffer, integer, fractional, fractional_digits);
        }

        if buffer.is_empty() {
            let error = match try_write_parts(buffer, integer, fractional, fractional_digits) {
                Ok(_) => unreachable!(),
                Err(error) => error,
            };
            return Err(BufferTooSmall {
                required: error.required + 1,
                provided: 0,
            });
        }

        match try_write_parts(&mut buffer[1..], integer, fractional, fractional_digits) {
            Ok(len) => {
                buffer[0] = b'-';
                Ok(len + 1)
            }
            Err(error) => Err(BufferTooSmall {
                required: error.required + 1,
                provided: buffer.len(),
            }),
        }
    }
}

impl_decimal_formatting!(DecimalU64);
impl_decimal_formatting!(DecimalI64);

#[inline]
fn try_write_parts(
    buffer: &mut [u8],
    integer: u64,
    mut fractional: u64,
    fractional_digits: usize,
) -> Result<usize, BufferTooSmall> {
    let integer_digits = if integer == 0 { 1 } else { integer.ilog10() as usize + 1 };
    let required = integer_digits
        + if fractional_digits == 0 {
            0
        } else {
            fractional_digits + 1
        };
    if buffer.len() < required {
        return Err(BufferTooSmall {
            required,
            provided: buffer.len(),
        });
    }

    let mut cursor = integer_digits;
    let mut integer = integer;

    while integer >= 100 {
        let pair = (integer % 100) as usize;
        integer /= 100;
        cursor -= 2;
        write_digit_pair(buffer, cursor, pair);
    }

    if integer < 10 {
        cursor -= 1;
        // SAFETY: the buffer length was checked above.
        unsafe {
            *buffer.get_unchecked_mut(cursor) = b'0' + integer as u8;
        }
    } else {
        cursor -= 2;
        write_digit_pair(buffer, cursor, integer as usize);
    }

    if fractional_digits == 0 {
        return Ok(integer_digits);
    }

    // SAFETY: the buffer length was checked above.
    unsafe {
        *buffer.get_unchecked_mut(integer_digits) = b'.';
    }

    let fractional_start = integer_digits + 1;
    let end = fractional_start + fractional_digits;
    cursor = end;
    let mut remaining = fractional_digits;

    while remaining >= 2 {
        let pair = (fractional % 100) as usize;
        fractional /= 100;
        cursor -= 2;
        write_digit_pair(buffer, cursor, pair);
        remaining -= 2;
    }

    if remaining != 0 {
        cursor -= 1;
        // SAFETY: the buffer length was checked above.
        unsafe {
            *buffer.get_unchecked_mut(cursor) = b'0' + fractional as u8;
        }
    }

    Ok(end)
}
