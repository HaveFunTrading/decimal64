#![warn(missing_docs)]

//! Fast unsigned fixed-point decimal arithmetic backed by a single [`u64`].
//!
//! [`DecimalU64`] stores an unscaled integer while its decimal scale is encoded
//! in the type. The crate provides the sealed scales [`U0`] through [`U8`], so
//! values with different scales are different Rust types and cannot be mixed
//! accidentally.
//!
//! # Example
//!
//! ```
//! use decimal64::{DecimalU64, U2, U8};
//!
//! let price = DecimalU64::<U2>::from_str("80500.32")?;
//! let quantity = DecimalU64::<U2>::from_str("2.00")?;
//! let total = price.checked_mul(quantity).unwrap();
//! assert_eq!("161000.64", total.to_string());
//!
//! let normalized = price.rescale::<U8>()?;
//! assert_eq!("80500.32000000", normalized.to_string());
//! # Ok::<(), decimal64::Error>(())
//! ```
//!
//! # Parsing and scaling
//!
//! [`DecimalU64::from_str`] and [`DecimalU64::from_slice`] accept plain,
//! non-negative decimal notation. Parsing is exact: fractional digits beyond
//! the target scale are accepted only when they are zero, and parsing never
//! rounds. Scientific notation is intentionally unsupported.
//!
//! [`DecimalU64::rescale`] converts between the built-in scales. Upscaling
//! returns [`Error::Overflow`] when the result does not fit in `u64`;
//! downscaling rounds half-up.
//!
//! # Arithmetic
//!
//! Arithmetic operands must have the same scale. The standard `+`, `-`, `*`,
//! and `/` operators panic on overflow, underflow, or division by zero in every
//! build profile. The corresponding `checked_*`, `saturating_*`, `wrapping_*`,
//! and `overflowing_*` method families provide explicit alternatives.
//! Multiplication and division truncate toward zero at the result scale.
//!
//! Values with different scales cannot be combined accidentally:
//!
//! ```compile_fail
//! use decimal64::{DecimalU64, U2, U4};
//!
//! let price = DecimalU64::<U2>::new(100);
//! let quantity = DecimalU64::<U4>::new(100);
//! let _ = price + quantity;
//! ```
//!
//! # Formatting
//!
//! [`DecimalU64::write_to`] and [`DecimalU64::write_to_trimmed`] write directly
//! into caller-provided buffers without allocation. [`Display`] and
//! `ToString::to_string` are also supported.
//!
//! # Optional features
//!
//! The `serde` feature serializes and deserializes all built-in scales as
//! decimal strings. Numeric tokens are rejected so deserialization follows the
//! same exact policy as string parsing.

use crate::macros::gen_scale;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

mod arithmetic;
pub mod error;
pub use error::{BufferTooSmall, Error, InvalidInputKind};
mod macros;
pub mod round;
#[cfg(feature = "serde")]
mod serde;

/// Scale metadata implemented by the built-in [`U0`] through [`U8`] scales.
///
/// This trait is sealed and cannot be implemented outside this crate.
///
/// ```compile_fail
/// use decimal64::ScaleMetrics;
///
/// struct U9;
///
/// impl ScaleMetrics for U9 {
///     const SCALE: u8 = 9;
///     const SCALE_FACTOR: u64 = 1_000_000_000;
///     const REQUIRED_BUFFER_LEN: usize = 21;
/// }
/// ```
pub trait ScaleMetrics: private::Sealed + Copy {
    /// Number of digits after the decimal point.
    const SCALE: u8;

    /// Power of ten used to convert between represented and unscaled values.
    const SCALE_FACTOR: u64;

    /// Buffer length sufficient to format any decimal with this scale.
    const REQUIRED_BUFFER_LEN: usize;
}

gen_scale!(U0, 0, 20);
gen_scale!(U1, 1, 21);
gen_scale!(U2, 2, 21);
gen_scale!(U3, 3, 21);
gen_scale!(U4, 4, 21);
gen_scale!(U5, 5, 21);
gen_scale!(U6, 6, 21);
gen_scale!(U7, 7, 21);
gen_scale!(U8, 8, 21);

const SCALE_FACTORS: [u64; 9] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000];
const POW5_U128: [u128; 9] = [1, 5, 25, 125, 625, 3125, 15625, 78125, 390625];
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

/// An unsigned fixed-point decimal with a compile-time scale.
///
/// `S` is one of the built-in [`U0`] through [`U8`] scale types. The raw
/// unscaled representation is available through the public tuple field.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DecimalU64<S>(
    /// Raw unscaled integer representation.
    pub u64,
    PhantomData<S>,
);

impl<S: ScaleMetrics> Default for DecimalU64<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<S: ScaleMetrics> Display for DecimalU64<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // A buffer large enough for our formatted value.
        let mut buf = [0u8; 21];
        let len = self.write_to(&mut buf);
        // Since we know our data is all ASCII, this is safe.
        let s = unsafe { std::str::from_utf8_unchecked(&buf[..len]) };
        f.write_str(s)
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Creates a decimal from its raw unscaled representation.
    ///
    /// The represented value is `unscaled / S::SCALE_FACTOR`. No parsing,
    /// rounding, or validation is performed.
    ///
    /// # Example
    ///
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::new(12_345);
    /// assert_eq!("123.45", value.to_string());
    /// ```
    #[inline]
    pub const fn new(unscaled: u64) -> Self {
        Self(unscaled, PhantomData)
    }

    /// The value zero.
    pub const ZERO: Self = DecimalU64::new(0);
    /// The smallest representable value.
    pub const MIN: Self = Self::ZERO;
    /// The value one.
    pub const ONE: Self = DecimalU64::new(S::SCALE_FACTOR);
    /// The value two.
    pub const TWO: Self = DecimalU64::new(2 * S::SCALE_FACTOR);
    /// The value three.
    pub const THREE: Self = DecimalU64::new(3 * S::SCALE_FACTOR);
    /// The value four.
    pub const FOUR: Self = DecimalU64::new(4 * S::SCALE_FACTOR);
    /// The value five.
    pub const FIVE: Self = DecimalU64::new(5 * S::SCALE_FACTOR);
    /// The value six.
    pub const SIX: Self = DecimalU64::new(6 * S::SCALE_FACTOR);
    /// The value seven.
    pub const SEVEN: Self = DecimalU64::new(7 * S::SCALE_FACTOR);
    /// The value eight.
    pub const EIGHT: Self = DecimalU64::new(8 * S::SCALE_FACTOR);
    /// The value nine.
    pub const NINE: Self = DecimalU64::new(9 * S::SCALE_FACTOR);
    /// The value ten.
    pub const TEN: Self = DecimalU64::new(10 * S::SCALE_FACTOR);
    /// The largest representable raw value at this scale.
    pub const MAX: Self = DecimalU64::new(u64::MAX);

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

        let input_len = bytes.len() - index;
        let scale = S::SCALE as usize;

        // Fixed-scale wire formats are common in market data. When the point
        // is exactly where the target scale requires it, parse both fields
        // independently and perform only two final overflow checks.
        if scale != 0 && input_len > scale {
            let decimal_point = bytes.len() - scale - 1;
            if bytes[decimal_point] == b'.' && decimal_point - index <= 19 {
                return Self::from_exact_scale_slice(bytes, index, decimal_point);
            }
        }

        // Any decimal with at most 19 effective digits fits in a `u64`. The
        // input length is a conservative upper bound for its digit count, so
        // these inputs need no overflow checks at all. `ScaleMetrics` is
        // sealed to U0–U8, making this subtraction safe.
        if input_len <= 19 - scale {
            return Self::from_short_slice::<false>(bytes, index);
        }

        // Up to 19 input digits also fit in the accumulator. Only the final
        // scale adjustment can overflow, reducing the hot loop to unchecked
        // arithmetic and a single checked multiplication.
        if input_len <= 19 {
            return Self::from_short_slice::<true>(bytes, index);
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

        Ok(Self(unscaled, PhantomData))
    }

    #[inline(always)]
    const fn from_exact_scale_slice(bytes: &[u8], index: usize, decimal_point: usize) -> Result<Self, Error> {
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
        Ok(Self(unscaled, PhantomData))
    }

    #[inline(always)]
    const fn from_short_slice<const CHECK_SCALE_OVERFLOW: bool>(bytes: &[u8], mut index: usize) -> Result<Self, Error> {
        if bytes.len() - index == 8
            && let Some(integer) = parse_eight_digits(bytes, index)
        {
            let unscaled = integer as u64 * S::SCALE_FACTOR;
            return Ok(Self(unscaled, PhantomData));
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
        Ok(Self(unscaled, PhantomData))
    }

    /// Parses an exactly representable decimal from a UTF-8 string slice.
    ///
    /// Fractional digits beyond the target scale are accepted only when they
    /// are zero. This method never rounds.
    ///
    /// # Example
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// assert_eq!("12.34", value.to_string());
    /// ```
    pub const fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_slice(s.as_bytes())
    }

    /// Converts this decimal to `f64`.
    ///
    /// # Examples
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// assert_eq!(12.34, value.to_f64());
    /// ```
    pub const fn to_f64(self) -> f64 {
        self.0 as f64 / S::SCALE_FACTOR as f64
    }

    /// Creates a decimal from `f64`, rounding half-up at the target scale.
    /// Returns error on invalid input or overflow.
    ///
    /// # Examples
    /// No rounding.
    /// ```
    /// use decimal64::{DecimalU64, U3};
    ///
    /// let value = DecimalU64::<U3>::from_f64(12.345).unwrap();
    /// assert_eq!("12.345", value.to_string());
    /// ```
    ///
    /// With rounding.
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::from_f64(12.345).unwrap();
    /// assert_eq!("12.35", value.to_string());
    /// ```
    pub const fn from_f64(value: f64) -> Result<Self, Error> {
        const EXP_BITS: u64 = 0x7ff;
        const EXP_BIAS: i32 = 1023;
        const MANTISSA_BITS: u32 = 52;
        const MANTISSA_MASK: u64 = (1u64 << MANTISSA_BITS) - 1;

        let bits = value.to_bits();
        let sign = bits >> 63;
        let exp_bits = ((bits >> MANTISSA_BITS) & EXP_BITS) as u16;
        let frac_bits = bits & MANTISSA_MASK;

        if exp_bits == EXP_BITS as u16 {
            let kind = if frac_bits == 0 {
                InvalidInputKind::InfiniteNumber
            } else {
                InvalidInputKind::NotANumber
            };
            return Err(Error::InvalidInput(kind));
        }
        if sign == 1 && (exp_bits != 0 || frac_bits != 0) {
            return Err(Error::InvalidInput(InvalidInputKind::NegativeNumber));
        }
        if exp_bits == 0 && frac_bits == 0 {
            return Ok(Self::ZERO);
        }

        let (mantissa, exp2) = if exp_bits == 0 {
            (frac_bits as u128, 1 - EXP_BIAS - MANTISSA_BITS as i32)
        } else {
            let mantissa = ((1u64 << MANTISSA_BITS) | frac_bits) as u128;
            (mantissa, exp_bits as i32 - EXP_BIAS - MANTISSA_BITS as i32)
        };

        // value = mantissa * 2^exp2; scaling by 10^S becomes 5^S * 2^(exp2 + S).
        let base = mantissa * POW5_U128[S::SCALE as usize];
        let exp2 = exp2 + S::SCALE as i32;

        if exp2 >= 0 {
            let shift = exp2 as u32;
            if shift >= 128 {
                return Err(Error::Overflow);
            }
            if base > (u64::MAX as u128 >> shift) {
                return Err(Error::Overflow);
            }
            let unscaled = base << shift;
            Ok(DecimalU64::new(unscaled as u64))
        } else {
            let shift = (-exp2) as u32;
            if shift >= 128 {
                // Denominator exceeds u128; remainder cannot reach half, so this rounds to zero.
                return Ok(Self::ZERO);
            }
            let denom = 1u128 << shift;
            let mut unscaled = base / denom;
            let remainder = base % denom;

            if remainder != 0 && (remainder << 1) >= denom {
                if unscaled == u64::MAX as u128 {
                    return Err(Error::Overflow);
                }
                unscaled += 1;
            }

            if unscaled > u64::MAX as u128 {
                return Err(Error::Overflow);
            }

            Ok(DecimalU64::new(unscaled as u64))
        }
    }

    /// Rescales this decimal to a different scale, returning an error on overflow.
    /// Downscaling rounds half-up when fractional digits are dropped.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] when the value cannot be represented at the
    /// target scale.
    ///
    /// # Example
    /// Scale up (will error on overflow).
    /// ```
    /// use decimal64::{DecimalU64, U2, U4};
    ///
    /// let amount = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// let upscaled = amount.rescale::<U4>().unwrap();
    /// assert_eq!("12.3400", upscaled.to_string());
    /// ```
    ///
    /// Scale down (rounds half-up).
    /// ```
    /// use decimal64::{DecimalU64, U2, U4};
    ///
    /// let amount = DecimalU64::<U4>::from_str("1.2050").unwrap();
    /// let downscaled = amount.rescale::<U2>().unwrap();
    /// assert_eq!("1.21", downscaled.to_string());
    /// ```
    #[inline]
    pub const fn rescale<T: ScaleMetrics>(self) -> Result<DecimalU64<T>, Error> {
        if T::SCALE == S::SCALE {
            Ok(DecimalU64::<T>::new(self.0))
        } else if T::SCALE > S::SCALE {
            let factor = SCALE_FACTORS[(T::SCALE - S::SCALE) as usize];
            let unscaled = match self.0.checked_mul(factor) {
                Some(value) => value,
                None => return Err(Error::Overflow),
            };

            Ok(DecimalU64::<T>::new(unscaled))
        } else {
            let factor = SCALE_FACTORS[(S::SCALE - T::SCALE) as usize];
            let truncated = self.0 / factor;
            let remainder = self.0 % factor;
            let rounded = if remainder != 0 && remainder >= factor / 2 {
                truncated + 1
            } else {
                truncated
            };
            Ok(DecimalU64::<T>::new(rounded))
        }
    }

    /// Split `unscaled` value into integer and fractional parts.
    ///
    /// # Example
    /// ```
    ///
    /// use decimal64::{DecimalU64, U6};
    ///
    /// let (int_part, frac_part) = DecimalU64::<U6>::from_str("123.45").unwrap().split();
    /// assert_eq!(123, int_part);
    /// assert_eq!(450000, frac_part);
    /// ```
    #[inline]
    pub const fn split(self) -> (u64, u64) {
        let integer_part = self.0 / S::SCALE_FACTOR;
        let fractional_part = self.0 % S::SCALE_FACTOR;
        (integer_part, fractional_part)
    }

    /// Writes this decimal into `buffer`, including trailing zeros, and returns
    /// the number of bytes written.
    ///
    /// Use [`Self::write_to_trimmed`] to omit trailing fractional zeros or
    /// [`Self::try_write_to`] to handle an insufficient buffer without panicking.
    ///
    /// # Panics
    ///
    /// Panics if `buffer` is too small for this value.
    ///
    /// # Examples
    ///
    /// ```
    /// use decimal64::{DecimalU64, U2};
    ///
    /// let value = DecimalU64::<U2>::from_str("1.2").unwrap();
    /// let mut buffer = [0u8; 4];
    /// let len = value.write_to(&mut buffer);
    /// assert_eq!("1.20", std::str::from_utf8(&buffer[..len]).unwrap());
    /// ```
    #[must_use]
    #[inline]
    pub fn write_to(&self, buffer: &mut [u8]) -> usize {
        match self.try_write_to(buffer) {
            Ok(len) => len,
            Err(error) => panic!("{error}"),
        }
    }

    /// Tries to write this decimal into `buffer`, including trailing zeros.
    ///
    /// # Errors
    ///
    /// Returns [`BufferTooSmall`] if `buffer` cannot hold this value. The
    /// buffer is not modified on error.
    #[inline]
    pub fn try_write_to(&self, buffer: &mut [u8]) -> Result<usize, BufferTooSmall> {
        let (integer, fractional) = self.split();
        Self::try_write_parts(buffer, integer, fractional, S::SCALE as usize)
    }

    /// Writes this decimal into `buffer` without trailing fractional zeros.
    ///
    /// Use [`Self::write_to`] to retain the full scale or
    /// [`Self::try_write_to_trimmed`] to handle an insufficient buffer without
    /// panicking.
    ///
    /// # Panics
    ///
    /// Panics if `buffer` is too small for the trimmed value.
    ///
    /// # Example
    ///
    /// ```
    /// use decimal64::{DecimalU64, U4};
    ///
    /// let value = DecimalU64::<U4>::from_str("12.3400").unwrap();
    /// let mut buffer = [0u8; 5];
    /// let len = value.write_to_trimmed(&mut buffer);
    /// assert_eq!("12.34", std::str::from_utf8(&buffer[..len]).unwrap());
    /// ```
    #[must_use]
    #[inline]
    pub fn write_to_trimmed(&self, buffer: &mut [u8]) -> usize {
        match self.try_write_to_trimmed(buffer) {
            Ok(len) => len,
            Err(error) => panic!("{error}"),
        }
    }

    /// Tries to write this decimal into `buffer` without trailing fractional
    /// zeros.
    ///
    /// # Errors
    ///
    /// Returns [`BufferTooSmall`] if `buffer` cannot hold the trimmed value.
    /// The buffer is not modified on error.
    #[inline]
    pub fn try_write_to_trimmed(&self, buffer: &mut [u8]) -> Result<usize, BufferTooSmall> {
        let (integer, mut fractional) = self.split();
        let mut fractional_digits = S::SCALE as usize;
        while fractional_digits != 0 && fractional % 10 == 0 {
            fractional /= 10;
            fractional_digits -= 1;
        }
        Self::try_write_parts(buffer, integer, fractional, fractional_digits)
    }

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
}

impl<S: ScaleMetrics> std::str::FromStr for DecimalU64<S> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_slice(s.as_bytes())
    }
}

impl<S: ScaleMetrics> TryFrom<f64> for DecimalU64<S> {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::from_f64(value)
    }
}

impl<S: ScaleMetrics> From<DecimalU64<S>> for f64 {
    fn from(value: DecimalU64<S>) -> Self {
        value.to_f64()
    }
}

mod private {
    pub trait Sealed {}
}

/// Parses eight validated ASCII digits using packed integer arithmetic.
#[inline(always)]
const fn parse_eight_digits(bytes: &[u8], index: usize) -> Option<u32> {
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
    use super::*;

    #[test]
    fn should_not_increase_size() {
        assert_eq!(std::mem::size_of::<u64>(), std::mem::size_of::<DecimalU64<U0>>());
        assert_eq!(std::mem::size_of::<u64>(), std::mem::size_of::<DecimalU64<U4>>());
        assert_eq!(std::mem::size_of::<u64>(), std::mem::size_of::<DecimalU64<U8>>());
    }

    #[test]
    fn should_parse_from_bytes() -> anyhow::Result<()> {
        assert_eq!(
            18446744073709551615,
            DecimalU64::<U0>::from_str("18446744073709551615")?.0
        );
        assert_eq!(
            18446744073709551615,
            DecimalU64::<U8>::from_str("184467440737.09551615")?.0
        );
        assert_eq!(12345000000, DecimalU64::<U8>::from_str("123.45000000")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123.")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123.0")?.0);
        assert_eq!(
            18446744073709551615,
            DecimalU64::<U8>::from_str("184467440737.09551615")?.0
        );
        assert_eq!(0, DecimalU64::<U8>::from_str("0.0")?.0);
        assert_eq!(0, DecimalU64::<U8>::from_str("0")?.0);
        Ok(())
    }

    #[test]
    fn should_parse_only_exactly_representable_values() -> anyhow::Result<()> {
        assert_eq!(
            DecimalU64::<U2>::from_str("1.23")?,
            DecimalU64::<U2>::from_str("1.2300")?
        );
        assert_eq!(DecimalU64::<U2>::from_str("0.50")?, DecimalU64::<U2>::from_str(".5")?);
        assert_eq!(
            DecimalU64::<U8>::from_str("0.12345678")?,
            DecimalU64::<U8>::from_str(".12345678")?
        );
        assert_eq!(
            DecimalU64::<U2>::from_str("1.23")?,
            DecimalU64::<U2>::from_str("+1.23")?
        );
        assert_eq!(
            DecimalU64::<U0>::MAX,
            DecimalU64::<U0>::from_str("18446744073709551615.000")?
        );
        assert_eq!(DecimalU64::<U0>::ONE, DecimalU64::<U0>::from_str("1.0")?);

        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss)),
            DecimalU64::<U2>::from_str("1.2301")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss)),
            DecimalU64::<U0>::from_str("1.1")
        );

        Ok(())
    }

    #[test]
    fn should_reject_malformed_input() {
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::Empty)),
            DecimalU64::<U2>::from_str("")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::MissingDigits)),
            DecimalU64::<U2>::from_str(".")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::MissingDigits)),
            DecimalU64::<U2>::from_str("+")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::MultipleDecimalPoints)),
            DecimalU64::<U2>::from_str("1.2.3")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber)),
            DecimalU64::<U2>::from_str("-1")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::InvalidByte(b'e'))),
            DecimalU64::<U2>::from_str("1e2")
        );
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::InvalidByte(0xff))),
            DecimalU64::<U2>::from_slice(&[b'1', 0xff])
        );
    }

    #[test]
    fn should_handle_more_than_u8_max_fractional_digits() {
        let exact = format!("0.{}", "0".repeat(300));
        assert_eq!(Ok(DecimalU64::<U8>::ZERO), DecimalU64::<U8>::from_str(&exact));

        let inexact = format!("0.{}1", "0".repeat(300));
        assert_eq!(
            Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss)),
            DecimalU64::<U8>::from_str(&inexact)
        );
    }

    #[test]
    fn should_implement_std_from_str() -> anyhow::Result<()> {
        let value: DecimalU64<U2> = "1.23".parse()?;
        assert_eq!(DecimalU64::<U2>::new(123), value);
        Ok(())
    }

    #[test]
    fn should_parse_packed_ascii_digits() {
        for expected in [0, 1, 12_345_678, 50_607_080, 99_999_999] {
            let mut digits = [b'0'; 8];
            let mut value = expected;
            let mut index = digits.len();
            while index != 0 {
                index -= 1;
                digits[index] += (value % 10) as u8;
                value /= 10;
            }
            assert_eq!(Some(expected), parse_eight_digits(&digits, 0));
        }

        for index in 0..8 {
            for invalid in 0..=u8::MAX {
                if invalid.is_ascii_digit() {
                    continue;
                }
                let mut digits = *b"12345678";
                digits[index] = invalid;
                assert_eq!(None, parse_eight_digits(&digits, 0));
            }
        }
    }

    #[test]
    fn should_round_trip_optimized_parsing_paths() {
        fn assert_round_trips<S: ScaleMetrics>() {
            for raw in [0, 1, S::SCALE_FACTOR, u64::MAX] {
                let encoded = DecimalU64::<S>::new(raw).to_string();
                assert_eq!(raw, DecimalU64::<S>::from_str(&encoded).unwrap().0);
            }

            let mut raw = 0x9e37_79b9_7f4a_7c15u64;
            for _ in 0..1_000 {
                raw = raw.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let encoded = DecimalU64::<S>::new(raw).to_string();
                assert_eq!(raw, DecimalU64::<S>::from_str(&encoded).unwrap().0);
            }
        }

        assert_round_trips::<U0>();
        assert_round_trips::<U1>();
        assert_round_trips::<U2>();
        assert_round_trips::<U3>();
        assert_round_trips::<U4>();
        assert_round_trips::<U5>();
        assert_round_trips::<U6>();
        assert_round_trips::<U7>();
        assert_round_trips::<U8>();
    }

    #[test]
    fn should_use_target_scale() -> anyhow::Result<()> {
        assert_eq!(12345600000, DecimalU64::<U8>::from_str("123.456")?.0);
        assert_eq!(1234560000, DecimalU64::<U7>::from_str("123.456")?.0);
        assert_eq!(123456000, DecimalU64::<U6>::from_str("123.456")?.0);
        assert_eq!(12345600, DecimalU64::<U5>::from_str("123.456")?.0);
        assert_eq!(1234560, DecimalU64::<U4>::from_str("123.456")?.0);
        assert_eq!(123456, DecimalU64::<U3>::from_str("123.456")?.0);
        assert!(DecimalU64::<U2>::from_str("123.456").is_err());
        assert!(DecimalU64::<U1>::from_str("123.456").is_err());
        assert!(DecimalU64::<U0>::from_str("123.456").is_err());
        Ok(())
    }

    #[test]
    fn should_split() -> anyhow::Result<()> {
        assert_eq!((123, 45000000), DecimalU64::<U8>::from_str("123.45000000")?.split());
        assert_eq!((0, 45000000), DecimalU64::<U8>::from_str("0.45000000")?.split());
        assert_eq!((0, 0), DecimalU64::<U8>::from_str("0.0")?.split());
        assert_eq!((123, 45000001), DecimalU64::<U8>::from_str("123.45000001")?.split());
        assert_eq!((123, 45100000), DecimalU64::<U8>::from_str("123.451")?.split());
        Ok(())
    }

    #[test]
    fn should_compare_for_eq() -> anyhow::Result<()> {
        let one = DecimalU64::<U8>::from_str("123.45000000")?;
        let two = DecimalU64::<U8>::from_str("123.45000000")?;
        assert_eq!(one, two);
        let one = DecimalU64::<U8>::from_str("123.45000000")?;
        let two = DecimalU64::<U8>::from_str("123.45000001")?;
        assert_ne!(one, two);
        let one = DecimalU64::<U8>::from_str("0.0")?;
        let two = DecimalU64::<U8>::from_str("0.0")?;
        assert_eq!(one, two);
        Ok(())
    }

    #[test]
    fn should_compare_for_ord() -> anyhow::Result<()> {
        let one = DecimalU64::<U8>::from_str("123.45000001")?;
        let two = DecimalU64::<U8>::from_str("123.45000000")?;
        assert!(one > two);
        let one = DecimalU64::<U8>::from_str("123.45000000")?;
        let two = DecimalU64::<U8>::from_str("123.45000001")?;
        assert!(one < two);
        let one = DecimalU64::<U8>::from_str("0.0")?;
        let two = DecimalU64::<U8>::from_str("0.0")?;
        assert!(one >= two);
        let one = DecimalU64::<U8>::from_str("0.0")?;
        let two = DecimalU64::<U8>::from_str("0.0")?;
        assert!(one <= two);
        Ok(())
    }

    #[test]
    fn should_err_if_number_too_large() {
        let err = DecimalU64::<U8>::from_str("184467440737.09551616");
        assert!(err.is_err());
        if let Err(err) = err {
            assert!(matches!(err, Error::Overflow));
        }
    }

    #[test]
    fn should_create_from_str() {
        assert_eq!(12345000001, DecimalU64::<U8>::from_str("123.45000001").unwrap().0);
    }

    #[test]
    fn should_error_on_from_f64_overflow() {
        let err = DecimalU64::<U0>::from_f64(1e30);
        assert!(matches!(err, Err(Error::Overflow)));
    }

    #[test]
    fn should_write_to_buffer() -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];

        let dec = DecimalU64::<U8>::from_str("123.45000001")?;
        assert_eq!(12, dec.write_to(&mut buf));
        assert_eq!("123.45000001", std::str::from_utf8(&buf[..12])?);

        let dec = DecimalU64::<U6>::from_str("123.45")?;
        assert_eq!(10, dec.write_to(&mut buf));
        assert_eq!("123.450000", std::str::from_utf8(&buf[..10])?);

        let dec = DecimalU64::<U0>::from_str("12345")?;
        assert_eq!(5, dec.write_to(&mut buf));
        assert_eq!("12345", std::str::from_utf8(&buf[..5])?);

        let dec = DecimalU64::<U0>::from_str("0")?;
        assert_eq!(1, dec.write_to(&mut buf));
        assert_eq!("0", std::str::from_utf8(&buf[..1])?);

        let dec = DecimalU64::<U8>::from_str("0")?;
        assert_eq!(10, dec.write_to(&mut buf));
        assert_eq!("0.00000000", std::str::from_utf8(&buf[..10])?);

        Ok(())
    }

    #[test]
    fn should_try_write_using_the_actual_output_length() {
        let value = DecimalU64::<U4>::from_str("12.3400").unwrap();

        let mut fixed_buffer = [0xa5; 5];
        let unchanged = fixed_buffer;
        assert_eq!(
            value.try_write_to(&mut fixed_buffer),
            Err(BufferTooSmall {
                required: 7,
                provided: 5,
            })
        );
        assert_eq!(fixed_buffer, unchanged);

        let mut trimmed_buffer = [0u8; 5];
        let len = value.try_write_to_trimmed(&mut trimmed_buffer).unwrap();
        assert_eq!("12.34", std::str::from_utf8(&trimmed_buffer[..len]).unwrap());
    }

    #[test]
    fn should_try_write_zero_to_its_actual_output_length() {
        let mut fixed_buffer = [0u8; 10];
        let len = DecimalU64::<U8>::ZERO.try_write_to(&mut fixed_buffer).unwrap();
        assert_eq!("0.00000000", std::str::from_utf8(&fixed_buffer[..len]).unwrap());

        let mut trimmed_buffer = [0u8; 1];
        let len = DecimalU64::<U8>::ZERO
            .try_write_to_trimmed(&mut trimmed_buffer)
            .unwrap();
        assert_eq!("0", std::str::from_utf8(&trimmed_buffer[..len]).unwrap());
    }

    #[test]
    fn should_display_to_string() -> anyhow::Result<()> {
        assert_eq!("123.450000", DecimalU64::<U6>::from_str("123.45")?.to_string());
        assert_eq!("123.45", DecimalU64::<U2>::from_str("123.45")?.to_string());
        assert_eq!("123.45000000", DecimalU64::<U8>::from_str("123.45")?.to_string());
        assert_eq!("0.00000000", DecimalU64::<U8>::from_str("0")?.to_string());
        assert_eq!("0", DecimalU64::<U0>::from_str("0")?.to_string());
        assert_eq!("10", DecimalU64::<U0>::from_str("10")?.to_string());
        Ok(())
    }

    #[test]
    fn should_default_to_zero() {
        assert_eq!("0.00000000", DecimalU64::<U8>::default().to_string());
        assert_eq!("0.0000000", DecimalU64::<U7>::default().to_string());
        assert_eq!("0.000000", DecimalU64::<U6>::default().to_string());
        assert_eq!("0.00000", DecimalU64::<U5>::default().to_string());
        assert_eq!("0.0000", DecimalU64::<U4>::default().to_string());
        assert_eq!("0.000", DecimalU64::<U3>::default().to_string());
        assert_eq!("0.00", DecimalU64::<U2>::default().to_string());
        assert_eq!("0.0", DecimalU64::<U1>::default().to_string());
        assert_eq!("0", DecimalU64::<U0>::default().to_string());
    }

    #[test]
    fn should_create_from_raw() {
        assert_eq!("0.00000123", DecimalU64::<U8>::new(123).to_string());
        assert_eq!("0.0000123", DecimalU64::<U7>::new(123).to_string());
        assert_eq!("123", DecimalU64::<U0>::new(123).to_string());
    }

    #[test]
    fn should_use_constant_zero() {
        assert_eq!("0.00000000", DecimalU64::<U8>::ZERO.to_string());
        assert_eq!("0.0000000", DecimalU64::<U7>::ZERO.to_string());
        assert_eq!("0.000000", DecimalU64::<U6>::ZERO.to_string());
        assert_eq!("0.00000", DecimalU64::<U5>::ZERO.to_string());
        assert_eq!("0.0000", DecimalU64::<U4>::ZERO.to_string());
        assert_eq!("0.000", DecimalU64::<U3>::ZERO.to_string());
        assert_eq!("0.00", DecimalU64::<U2>::ZERO.to_string());
        assert_eq!("0.0", DecimalU64::<U1>::ZERO.to_string());
        assert_eq!("0", DecimalU64::<U0>::ZERO.to_string());
    }

    #[test]
    fn should_use_constant_min() {
        assert_eq!(DecimalU64::<U0>::MIN, DecimalU64::ZERO);
        assert_eq!(DecimalU64::<U8>::MIN, DecimalU64::ZERO);
    }

    #[test]
    fn should_use_constant_one() {
        assert_eq!("1.00000000", DecimalU64::<U8>::ONE.to_string());
        assert_eq!("1.0000000", DecimalU64::<U7>::ONE.to_string());
        assert_eq!("1.000000", DecimalU64::<U6>::ONE.to_string());
        assert_eq!("1.00000", DecimalU64::<U5>::ONE.to_string());
        assert_eq!("1.0000", DecimalU64::<U4>::ONE.to_string());
        assert_eq!("1.000", DecimalU64::<U3>::ONE.to_string());
        assert_eq!("1.00", DecimalU64::<U2>::ONE.to_string());
        assert_eq!("1.0", DecimalU64::<U1>::ONE.to_string());
        assert_eq!("1", DecimalU64::<U0>::ONE.to_string());
    }

    #[test]
    fn should_use_constant_three() {
        assert_eq!("3.00000000", DecimalU64::<U8>::THREE.to_string());
        assert_eq!("3.0000000", DecimalU64::<U7>::THREE.to_string());
        assert_eq!("3.000000", DecimalU64::<U6>::THREE.to_string());
        assert_eq!("3.00000", DecimalU64::<U5>::THREE.to_string());
        assert_eq!("3.0000", DecimalU64::<U4>::THREE.to_string());
        assert_eq!("3.000", DecimalU64::<U3>::THREE.to_string());
        assert_eq!("3.00", DecimalU64::<U2>::THREE.to_string());
        assert_eq!("3.0", DecimalU64::<U1>::THREE.to_string());
        assert_eq!("3", DecimalU64::<U0>::THREE.to_string());
    }

    #[test]
    fn should_use_constant_max() {
        assert_eq!("184467440737.09551615", DecimalU64::<U8>::MAX.to_string());
        assert_eq!("1844674407370.9551615", DecimalU64::<U7>::MAX.to_string());
        assert_eq!("18446744073709.551615", DecimalU64::<U6>::MAX.to_string());
        assert_eq!("184467440737095.51615", DecimalU64::<U5>::MAX.to_string());
        assert_eq!("1844674407370955.1615", DecimalU64::<U4>::MAX.to_string());
        assert_eq!("18446744073709551.615", DecimalU64::<U3>::MAX.to_string());
        assert_eq!("184467440737095516.15", DecimalU64::<U2>::MAX.to_string());
        assert_eq!("1844674407370955161.5", DecimalU64::<U1>::MAX.to_string());
        assert_eq!("18446744073709551615", DecimalU64::<U0>::MAX.to_string());
    }

    #[test]
    fn should_write_max_to_buffer() {
        fn write_max<S: ScaleMetrics>(buffer: &mut [u8], value: DecimalU64<S>) -> usize {
            value.write_to(buffer)
        }

        let mut buffer = [0u8; 1024];

        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U8>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U7>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U6>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U5>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U4>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U3>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U2>::MAX));
        assert_eq!(21, write_max(&mut buffer, DecimalU64::<U1>::MAX));
        assert_eq!(20, write_max(&mut buffer, DecimalU64::<U0>::MAX));
    }

    #[test]
    #[should_panic(expected = "provided buffer length 1 is too small, requires at least 21 bytes")]
    fn should_panic_if_buffer_too_small() {
        let mut buffer = [0u8; 1];
        let _ = DecimalU64::<U8>::MAX.write_to(&mut buffer);
    }

    #[test]
    fn should_write_if_buffer_is_of_exact_size() {
        let mut buffer = [0u8; U8::REQUIRED_BUFFER_LEN];
        let _ = DecimalU64::<U8>::MAX.write_to(&mut buffer);
    }
}

#[cfg(test)]
mod rescale_tests {
    use crate::error::Error;
    use crate::{DecimalU64, ScaleMetrics, U0, U1, U2, U3, U4, U5, U7, U8};
    use rstest::rstest;

    // Generic rescale test for checked rescale when no rounding is needed.
    fn rescale<S1: ScaleMetrics, S2: ScaleMetrics>(s: &'static str) {
        let s1 = DecimalU64::<S1>::from_str(s).unwrap();
        let s2 = s1.rescale::<S2>().unwrap();

        // Compare decimal strings ignoring trailing zeros
        assert_eq!(
            s1.to_string().trim_end_matches('0').trim_end_matches('.'),
            s2.to_string().trim_end_matches('0').trim_end_matches('.')
        );
    }

    #[rstest]
    #[case("0")]
    #[case("1")]
    #[case("0.01")]
    #[case("1.25")]
    #[case("123.45")]
    fn should_rescale_up(#[case] s: &'static str) {
        rescale::<U2, U5>(s);
        rescale::<U2, U8>(s);
        rescale::<U3, U5>(s);
        rescale::<U5, U8>(s);
    }

    #[rstest]
    #[case("0")]
    #[case("1")]
    #[case("10")]
    #[case("123")]
    #[case("1.20")]
    #[case("123.450")]
    fn should_rescale_down(#[case] s: &'static str) {
        rescale::<U8, U5>(s);
        rescale::<U8, U2>(s);
        rescale::<U5, U2>(s);
        rescale::<U7, U4>(s);
    }

    #[rstest]
    #[case("50", "50")]
    #[case("12345", "12345")]
    fn should_not_rescale_with_same_base(#[case] s: &'static str, #[case] expected: &str) {
        let d = DecimalU64::<U4>::from_str(s).unwrap();
        let res = d.rescale::<U4>().unwrap();

        // Compare decimal strings ignoring trailing zeros
        assert_eq!(res.to_string().trim_end_matches('0').trim_end_matches('.'), expected);
    }

    #[rstest]
    #[case("12345")]
    #[case("123400")]
    fn should_round_trip_invariant(#[case] s: &'static str) {
        let d = DecimalU64::<U2>::from_str(s).unwrap();
        let up: DecimalU64<U8> = d.rescale().unwrap();
        let down: DecimalU64<U2> = up.rescale().unwrap();

        // Compare decimal values, not unscaled
        assert_eq!(
            d.to_string().trim_end_matches('0'),
            down.to_string().trim_end_matches('0')
        );
    }

    #[rstest]
    #[case("1.234", "1.23")]
    #[case("1.235", "1.24")]
    #[case("1.236", "1.24")]
    #[case("9.995", "10.00")]
    #[case("0.004", "0.00")]
    #[case("0.005", "0.01")]
    fn should_round_downscale_u3_to_u2(#[case] input: &str, #[case] expected: &str) {
        let d = DecimalU64::<U3>::from_str(input).unwrap();
        let result: DecimalU64<U2> = d.rescale().unwrap();
        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn should_round_downscale_u8_to_u0() {
        let d = DecimalU64::<U8>::from_str("1.50000000").unwrap();
        let result: DecimalU64<U0> = d.rescale().unwrap();
        assert_eq!("2", result.to_string());

        let d = DecimalU64::<U8>::from_str("1.49999999").unwrap();
        let result: DecimalU64<U0> = d.rescale().unwrap();
        assert_eq!("1", result.to_string());
    }

    #[test]
    fn should_round_on_downscale() {
        let d = DecimalU64::<U4>::from_str("101.2038").unwrap(); // 4 decimal places
        let result = d.rescale::<U2>().unwrap(); // Downscale to 2 decimals
        assert_eq!("101.20", result.to_string());

        let d = DecimalU64::<U4>::from_str("101.2050").unwrap(); // exactly half
        let result = d.rescale::<U2>().unwrap();
        assert_eq!("101.21", result.to_string());
    }

    #[test]
    fn should_error_on_overflow() {
        // Try to upscale MAX value at U0 to U1 (would multiply by 10, causing overflow)
        let d = DecimalU64::<U0>::MAX;
        let result: Result<DecimalU64<U1>, Error> = d.rescale();

        assert!(result.is_err());
        match result {
            Err(Error::Overflow) => {}
            _ => panic!("Expected Overflow error"),
        }
    }

    #[test]
    fn should_handle_upscale_overflow_boundary() {
        let max_ok = DecimalU64::<U0>::new(u64::MAX / 10);
        let up: DecimalU64<U1> = max_ok.rescale().unwrap();
        assert_eq!(max_ok.0 * 10, up.0);

        let too_big = DecimalU64::<U0>::new(u64::MAX / 10 + 1);
        let result: Result<DecimalU64<U1>, Error> = too_big.rescale();
        assert!(matches!(result, Err(Error::Overflow)));
    }

    #[test]
    fn should_rescale_in_const_contexts() {
        const UPSCALED: Result<DecimalU64<U4>, Error> = DecimalU64::<U2>::new(123).rescale();
        const DOWNSCALED: Result<DecimalU64<U2>, Error> = DecimalU64::<U3>::new(1235).rescale();

        assert_eq!(DecimalU64::<U4>::new(12_300), UPSCALED.unwrap());
        assert_eq!(DecimalU64::<U2>::new(124), DOWNSCALED.unwrap());
    }
}

#[cfg(test)]
mod f64_tests {
    use crate::error::{Error, InvalidInputKind};
    use crate::{DecimalU64, U2};

    #[test]
    fn should_convert_to_f64() {
        const VALUE: f64 = DecimalU64::<U2>::new(1234).to_f64();
        assert!((VALUE - 12.34).abs() < f64::EPSILON);
    }

    #[test]
    fn should_create_from_f64() -> anyhow::Result<()> {
        let dec = DecimalU64::<U2>::from_f64(12.25)?;
        assert_eq!("12.25", dec.to_string());
        Ok(())
    }

    #[test]
    fn should_convert_using_standard_traits() -> anyhow::Result<()> {
        let decimal = DecimalU64::<U2>::try_from(12.25)?;
        let value = f64::from(decimal);
        assert_eq!(12.25, value);
        Ok(())
    }

    #[test]
    fn should_create_from_f64_as_const() {
        const VALUE: Result<DecimalU64<U2>, Error> = DecimalU64::from_f64(12.25);
        assert_eq!("12.25", VALUE.unwrap().to_string());
    }

    #[test]
    fn should_round_from_f64_half_up() -> anyhow::Result<()> {
        let dec = DecimalU64::<U2>::from_f64(0.125)?;
        assert_eq!("0.13", dec.to_string());
        Ok(())
    }

    #[test]
    fn should_error_on_from_f64_infinity() {
        let err = DecimalU64::<U2>::from_f64(f64::INFINITY);
        assert!(matches!(
            err,
            Err(Error::InvalidInput(InvalidInputKind::InfiniteNumber))
        ));
    }

    #[test]
    fn should_error_on_from_f64_nan() {
        let err = DecimalU64::<U2>::from_f64(f64::NAN);
        assert!(matches!(err, Err(Error::InvalidInput(InvalidInputKind::NotANumber))));
    }

    #[test]
    fn should_error_on_from_f64_negative() {
        let err = DecimalU64::<U2>::from_f64(-1.0);
        assert!(matches!(
            err,
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
        ));
    }
}
