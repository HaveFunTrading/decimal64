//! Floating-point, scale, and signedness conversions.

use crate::scale::SCALE_FACTORS;
use crate::{DecimalI64, DecimalU64, Error, InvalidInputKind, ScaleMetrics};

const POW5_U128: [u128; 9] = [1, 5, 25, 125, 625, 3125, 15625, 78125, 390625];

const fn positive_f64_to_magnitude<S: ScaleMetrics>(value: f64) -> Result<u64, Error> {
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
        return Ok(0);
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
        Ok(unscaled as u64)
    } else {
        let shift = (-exp2) as u32;
        if shift >= 128 {
            // Denominator exceeds u128; remainder cannot reach half, so this rounds to zero.
            return Ok(0);
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

        Ok(unscaled as u64)
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
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
        match positive_f64_to_magnitude::<S>(value) {
            Ok(unscaled) => Ok(Self::new(unscaled)),
            Err(error) => Err(error),
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
}

impl<S: ScaleMetrics> DecimalI64<S> {
    /// Creates a decimal from a finite `f64`, rounding halfway cases away
    /// from zero at the target scale.
    ///
    /// Negative zero is normalized to zero.
    pub const fn from_f64(value: f64) -> Result<Self, Error> {
        const SIGN_MASK: u64 = 1u64 << 63;
        let bits = value.to_bits();
        let negative = bits & SIGN_MASK != 0;
        let magnitude_value = if negative { -value } else { value };
        let magnitude = match positive_f64_to_magnitude::<S>(magnitude_value) {
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

    /// Rescales this decimal, returning an error if the result is not
    /// representable. Downscaling rounds halfway cases away from zero.
    #[inline]
    pub const fn rescale<T: ScaleMetrics>(self) -> Result<DecimalI64<T>, Error> {
        if T::SCALE == S::SCALE {
            Ok(DecimalI64::<T>::new(self.0))
        } else if T::SCALE > S::SCALE {
            let factor = SCALE_FACTORS[(T::SCALE - S::SCALE) as usize] as i128;
            let unscaled = self.0 as i128 * factor;
            if unscaled < i64::MIN as i128 || unscaled > i64::MAX as i128 {
                Err(Error::Overflow)
            } else {
                Ok(DecimalI64::<T>::new(unscaled as i64))
            }
        } else {
            let factor = SCALE_FACTORS[(S::SCALE - T::SCALE) as usize] as i64;
            let truncated = self.0 / factor;
            let remainder = self.0 % factor;
            let rounded = if remainder.unsigned_abs() >= (factor as u64 / 2) {
                if self.0 < 0 { truncated - 1 } else { truncated + 1 }
            } else {
                truncated
            };
            Ok(DecimalI64::<T>::new(rounded))
        }
    }
}

impl_decimal_float_conversions!(DecimalU64);
impl_decimal_float_conversions!(DecimalI64);

impl<S: ScaleMetrics> TryFrom<DecimalU64<S>> for DecimalI64<S> {
    type Error = Error;

    fn try_from(value: DecimalU64<S>) -> Result<Self, Self::Error> {
        if value.0 > i64::MAX as u64 {
            Err(Error::Overflow)
        } else {
            Ok(Self::new(value.0 as i64))
        }
    }
}

impl<S: ScaleMetrics> TryFrom<DecimalI64<S>> for DecimalU64<S> {
    type Error = Error;

    fn try_from(value: DecimalI64<S>) -> Result<Self, Self::Error> {
        if value.0 < 0 {
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
        } else {
            Ok(Self::new(value.0 as u64))
        }
    }
}
