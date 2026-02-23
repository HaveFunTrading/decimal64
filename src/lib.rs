use crate::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;

mod arithmetic;
pub mod error;
mod macros;
pub mod round;
#[cfg(feature = "serde")]
pub mod serde;

pub trait ScaleMetrics {
    const SCALE: u8;
    const SCALE_FACTOR: u64;
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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DecimalU64<S>(pub u64, PhantomData<S>);

impl<S: ScaleMetrics> FromStr for DecimalU64<S> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl<S: ScaleMetrics> TryFrom<&[u8]> for DecimalU64<S> {
    type Error = Error;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut unscaled: u64 = 0;
        let mut fractional_part_flag = 0;
        let mut scale_counter = 0;

        for &byte in bytes {
            match byte {
                b'0'..=b'9' => {
                    unscaled = (unscaled * 10)
                        .checked_add((byte - b'0') as u64)
                        .ok_or_else(|| Error::Overflow(String::from_utf8_lossy(bytes).to_string()))?;

                    scale_counter += fractional_part_flag;
                }
                b'.' => fractional_part_flag = 1,
                other => return Err(Error::InvalidCharacterInput(other as char)),
            }
        }

        let unscaled = unscaled
            .checked_mul(*unsafe {
                SCALE_FACTORS.get_unchecked(
                    S::SCALE
                        .checked_sub(scale_counter)
                        .ok_or_else(|| Error::Overflow(String::from_utf8_lossy(bytes).to_string()))?
                        as usize,
                )
            })
            .ok_or_else(|| Error::Overflow(String::from_utf8_lossy(bytes).to_string()))?;

        Ok(Self(unscaled, PhantomData))
    }
}

impl<S: ScaleMetrics> Display for DecimalU64<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // A buffer large enough for our formatted value.
        let mut buf = [0u8; 64];
        let len = self.write_to(&mut buf);
        // Since we know our data is all ASCII, this is safe.
        let s = unsafe { std::str::from_utf8_unchecked(&buf[..len]) };
        f.write_str(s)
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    #[inline]
    pub const fn new(unscaled: u64) -> Self {
        Self(unscaled, PhantomData)
    }

    pub const ZERO: Self = DecimalU64::new(0);
    pub const ONE: Self = DecimalU64::new(S::SCALE_FACTOR);
    pub const TWO: Self = DecimalU64::new(2 * S::SCALE_FACTOR);
    pub const THREE: Self = DecimalU64::new(3 * S::SCALE_FACTOR);
    pub const FOUR: Self = DecimalU64::new(4 * S::SCALE_FACTOR);
    pub const FIVE: Self = DecimalU64::new(5 * S::SCALE_FACTOR);
    pub const SIX: Self = DecimalU64::new(6 * S::SCALE_FACTOR);
    pub const SEVEN: Self = DecimalU64::new(7 * S::SCALE_FACTOR);
    pub const EIGHT: Self = DecimalU64::new(8 * S::SCALE_FACTOR);
    pub const NINE: Self = DecimalU64::new(9 * S::SCALE_FACTOR);
    pub const TEN: Self = DecimalU64::new(10 * S::SCALE_FACTOR);
    pub const MAX: Self = DecimalU64::new(u64::MAX);

    /// Rescales this decimal to a different scale **without checking for overflow
    /// or precision loss**.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - Upscaling does not overflow `u64`
    /// - The resulting value is a valid `DecimalU64<T>`
    /// - Any precision loss caused by downscaling is acceptable
    ///
    /// # Example
    /// No precision loss.
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, U2, U4};
    ///
    /// let amount = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// let upscaled: DecimalU64<U4> = unsafe { amount.rescale_unchecked() };
    /// assert_eq!("12.3400", upscaled.to_string());
    /// ```
    ///
    /// Precision loss.
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, U2, U4};
    ///
    /// let amount = DecimalU64::<U4>::from_str("1.2399").unwrap();
    /// let downscaled: DecimalU64<U2> = unsafe { amount.rescale_unchecked() };
    /// assert_eq!("1.23", downscaled.to_string());
    /// ```
    pub const unsafe fn rescale_unchecked<T: ScaleMetrics>(&self) -> DecimalU64<T> {
        if T::SCALE >= S::SCALE {
            // Upscale: multiply
            let factor = 10u64.pow((T::SCALE - S::SCALE) as u32);
            DecimalU64::<T>::new(self.0.saturating_mul(factor))
        } else {
            // Downscale: divide (truncate)
            let factor = 10u64.pow((S::SCALE - T::SCALE) as u32);
            DecimalU64::<T>::new(self.0 / factor)
        }
    }

    /// Rescales this decimal to a different scale, returning an error on overflow
    /// or precision loss.
    ///
    /// # Example
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, U2, U4};
    ///
    /// let amount = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// let upscaled = amount.rescale::<U4>().unwrap();
    /// assert_eq!("12.3400", upscaled.to_string());
    /// ```
    pub fn rescale<T: ScaleMetrics>(&self) -> Result<DecimalU64<T>, self::Error> {
        if T::SCALE >= S::SCALE {
            // Upscale
            let factor = 10u64.checked_pow((T::SCALE - S::SCALE) as u32).ok_or_else(|| {
                Error::Overflow(format!("unable to upscale {} from {} to {}", self, S::SCALE, T::SCALE))
            })?;

            let unscaled = self.0.checked_mul(factor).ok_or_else(|| {
                Error::Overflow(format!("unable to upscale {} from {} to {}", self, S::SCALE, T::SCALE))
            })?;

            Ok(DecimalU64::<T>::new(unscaled))
        } else {
            // Downscale
            let factor = 10u64.checked_pow((S::SCALE - T::SCALE) as u32).ok_or_else(|| {
                Error::Overflow(format!("unable to downscale {} from {} to {}", self, S::SCALE, T::SCALE))
            })?;

            let truncated = self.0 / factor;
            let remainder = self.0 % factor;

            if remainder != 0 {
                // Precision loss occurred
                Err(Error::PrecisionLoss(format!(
                    "truncated {} fractional digits when rescaling {} -> {}",
                    S::SCALE - T::SCALE,
                    self.0,
                    truncated
                )))
            } else {
                Ok(DecimalU64::<T>::new(truncated))
            }
        }
    }

    /// Split `unscaled` value into integer and fractional parts.
    ///
    /// # Example
    /// ```no_run
    ///
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, U6};
    ///
    /// let (int_part, frac_part) = DecimalU64::<U6>::from_str("123.45").unwrap().split();
    /// assert_eq!(123, int_part);
    /// assert_eq!(450000, frac_part);
    /// ```
    #[inline]
    pub const fn split(&self) -> (u64, u64) {
        let integer_part = self.0 / S::SCALE_FACTOR;
        let fractional_part = self.0 % S::SCALE_FACTOR;
        (integer_part, fractional_part)
    }

    #[inline]
    /// Writes this decimal into `buffer` and returns the number of bytes written.  The buffer must
    /// be at least `S::REQUIRED_BUFFER_LEN` bytes. Output includes trailing zeros to match the scale.
    ///
    /// If you required trimmed output, use [`Self::write_to_trimmed`].
    ///
    /// ## Examples
    /// No trailing zeroes.
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, ScaleMetrics, U2};
    ///
    /// let value = DecimalU64::<U2>::from_str("12.34").unwrap();
    /// let mut buffer = [0u8; U2::REQUIRED_BUFFER_LEN];
    /// let len = value.write_to(&mut buffer);
    /// assert_eq!("12.34", std::str::from_utf8(&buffer[..len]).unwrap());
    /// ```
    /// With trailing zeroes
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, ScaleMetrics, U2};
    ///
    /// let value = DecimalU64::<U2>::from_str("1.2").unwrap();
    /// let mut buffer = [0u8; U2::REQUIRED_BUFFER_LEN];
    /// let len = value.write_to(&mut buffer);
    /// assert_eq!("1.20", std::str::from_utf8(&buffer[..len]).unwrap());
    /// ```
    pub fn write_to(&self, buffer: &mut [u8]) -> usize {
        #[cold]
        #[inline(never)]
        fn insufficient_buffer_len(len: usize, required: usize) -> ! {
            panic!("provided buffer length {} is too small, requires at least {} bytes", len, required);
        }

        // ensure the provided buffer has enough length to write the max value
        if S::REQUIRED_BUFFER_LEN > buffer.len() {
            insufficient_buffer_len(buffer.len(), S::REQUIRED_BUFFER_LEN)
        }

        // Compute the scale factor: 10^PRECISION.
        let (int_part, frac_part) = self.split();
        let mut pos = 0;

        // Write the integer part.
        if int_part == 0 {
            // SAFETY we already checked the destination buffer is of sufficient size
            unsafe {
                *buffer.get_unchecked_mut(pos) = b'0';
            }
            pos += 1;
        } else {
            let mut tmp = int_part;
            let mut digit_count = 0;
            while tmp != 0 {
                digit_count += 1;
                tmp /= 10;
            }
            pos += digit_count;
            let mut idx = pos;
            tmp = int_part;
            while tmp != 0 {
                idx -= 1;
                // SAFETY we already checked the destination buffer is of sufficient size
                unsafe {
                    *buffer.get_unchecked_mut(idx) = b'0' + (tmp % 10) as u8;
                }
                tmp /= 10;
            }
        }

        // If there is a fractional part, write the decimal point and fractional digits.
        if S::SCALE > 0 {
            // SAFETY we already checked the destination buffer is of sufficient size
            unsafe {
                *buffer.get_unchecked_mut(pos) = b'.';
            }
            pos += 1;
            // Start with the highest power of 10 for the fractional part.
            let mut divisor = 10u64.pow((S::SCALE - 1) as u32);
            let mut frac = frac_part;
            for _ in 0..S::SCALE {
                let digit = frac / divisor;
                // SAFETY we already checked the destination buffer is of sufficient size
                unsafe {
                    *buffer.get_unchecked_mut(pos) = b'0' + (digit as u8);
                }
                pos += 1;
                frac %= divisor;
                divisor /= 10;
            }
        }

        pos
    }

    /// Writes this decimal into `buffer` without trailing fractional zeros.
    ///
    /// If you required untrimmed output, use [`Self::write_to`].
    ///
    /// # Example
    /// ```no_run
    /// use std::str::FromStr;
    /// use decimal64::{DecimalU64, U4};
    ///
    /// let value = DecimalU64::<U4>::from_str("12.3400").unwrap();
    /// let mut buffer = [0u8; 32];
    /// let len = value.write_to_trimmed(&mut buffer);
    /// assert_eq!("12.34", std::str::from_utf8(&buffer[..len]).unwrap());
    /// ```
    pub fn write_to_trimmed(&self, buffer: &mut [u8]) -> usize {
        let len = self.write_to(buffer);
        if S::SCALE == 0 {
            return len;
        }

        let mut end = len;
        while end > 0 {
            // SAFETY: end > 0 and end <= len <= buffer.len()
            let byte = unsafe { *buffer.get_unchecked(end - 1) };
            if byte != b'0' {
                break;
            }
            end -= 1;
        }
        if end > 0 {
            // SAFETY: end > 0 and end <= len <= buffer.len()
            let byte = unsafe { *buffer.get_unchecked(end - 1) };
            if byte == b'.' {
                end -= 1;
            }
        }

        end
    }
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
        assert_eq!(18446744073709551615, DecimalU64::<U0>::from_str("18446744073709551615")?.0);
        assert_eq!(18446744073709551615, DecimalU64::<U8>::from_str("184467440737.09551615")?.0);
        assert_eq!(12345000000, DecimalU64::<U8>::from_str("123.45000000")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123.")?.0);
        assert_eq!(12300000000, DecimalU64::<U8>::from_str("123.0")?.0);
        assert_eq!(18446744073709551615, DecimalU64::<U8>::from_str("184467440737.09551615")?.0);
        assert_eq!(0, DecimalU64::<U8>::from_str("0.0")?.0);
        assert_eq!(0, DecimalU64::<U8>::from_str("0")?.0);
        Ok(())
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
            assert_eq!("overflow: 184467440737.09551616", err.to_string());
        }
    }

    #[test]
    fn should_create_from_str() {
        assert_eq!(12345000001, DecimalU64::<U8>::from_str("123.45000001").unwrap().0);
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
        DecimalU64::<U8>::MAX.write_to(&mut buffer);
    }

    #[test]
    fn should_write_if_buffer_is_of_exact_size() {
        let mut buffer = [0u8; U8::REQUIRED_BUFFER_LEN];
        DecimalU64::<U8>::MAX.write_to(&mut buffer);
    }
}

#[cfg(test)]
mod rescale_tests {
    use crate::error::Error;
    use crate::{DecimalU64, ScaleMetrics, U0, U1, U2, U3, U4, U5, U7, U8};
    use rstest_macros::rstest;
    use std::str::FromStr;

    // Generic rescale test for checked rescale (exact)
    fn rescale<S1: ScaleMetrics, S2: ScaleMetrics>(s: &'static str) {
        let s1 = DecimalU64::<S1>::from_str(s).unwrap();
        let s2 = s1.rescale::<S2>().unwrap();

        // Compare decimal strings ignoring trailing zeros
        assert_eq!(
            s1.to_string().trim_end_matches('0').trim_end_matches('.'),
            s2.to_string().trim_end_matches('0').trim_end_matches('.')
        );
    }

    // Generic unchecked rescale test - compare the actual decimal value
    fn rescale_unchecked<S1: ScaleMetrics, S2: ScaleMetrics>(s: &'static str, expected: &str) {
        let d = DecimalU64::<S1>::from_str(s).unwrap();
        let res: DecimalU64<S2> = unsafe { d.rescale_unchecked() };
        assert_eq!(res.to_string(), expected); // Compare Display output, not unscaled
    }

    // -------------------------
    // RESCALE UP (checked)
    // -------------------------
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
    #[case("0", "0.00000000")]
    #[case("1", "1.00000000")] // U0 -> U8: 1 becomes 1.00000000
    #[case("12", "12.00000000")] // U0 -> U8: 12 becomes 12.00000000
    #[case("1234", "1234.00000000")] // U0 -> U8: 1234 becomes 1234.00000000
    #[case("999999", "999999.00000000")] // U0 -> U8: 999999 becomes 999999.00000000
    fn should_upscale_unchecked_u0_to_u8(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U0, U8>(s, expected);
    }

    #[rstest]
    #[case("1.23", "1.23000000")] // U2 -> U8: 1.23 becomes 1.23000000
    #[case("12.34", "12.34000000")] // U2 -> U8: 12.34 becomes 12.34000000
    #[case("123.45", "123.45000000")] // U2 -> U8: 123.45 becomes 123.45000000
    #[case("999.99", "999.99000000")] // U2 -> U8: 999.99 becomes 999.99000000
    fn should_upscale_unchecked_u2_to_u8(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U2, U8>(s, expected);
    }

    #[rstest]
    #[case("1.2345", "1.23450000")] // U4 -> U8: 1.2345 becomes 1.23450000
    #[case("12.3456", "12.34560000")] // U4 -> U8: 12.3456 becomes 12.34560000
    fn should_upscale_unchecked_u4_to_u8(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U4, U8>(s, expected);
    }

    #[rstest]
    #[case("1.20000000", "1.20")] // U8 -> U2: 1.20000000 becomes 1.20
    #[case("123.40000000", "123.40")] // U8 -> U2: 123.40000000 becomes 123.40
    #[case("0.50000000", "0.50")] // U8 -> U2: 0.50000000 becomes 0.50
    #[case("0.99000000", "0.99")] // U8 -> U2: 0.99000000 becomes 0.99
    #[case("123.45678900", "123.45")] // U8 -> U2: 123.45678900 becomes 123.45 (truncated)
    fn should_downscale_unchecked_u8_to_u2(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U8, U2>(s, expected);
    }

    #[rstest]
    #[case("1.20000000", "1.200")] // U8 -> U3: 1.20000000 becomes 1.200
    #[case("123.45678900", "123.456")] // U8 -> U3: 123.45678900 becomes 123.456 (truncated)
    fn should_downscale_unchecked_u8_to_u3(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U8, U3>(s, expected);
    }

    #[rstest]
    #[case("1.20000000", "1.2")] // U8 -> U1: 1.20000000 becomes 1.2
    #[case("123.40000000", "123.4")] // U8 -> U1: 123.40000000 becomes 123.4
    fn should_downscale_unchecked_u8_to_u1(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U8, U1>(s, expected);
    }

    #[rstest]
    #[case("1.20000000", "1")] // U8 -> U0: 1.20000000 becomes 1
    #[case("123.40000000", "123")] // U8 -> U0: 123.40000000 becomes 123
    #[case("0.50000000", "0")] // U8 -> U0: 0.50000000 becomes 0 (truncated)
    fn should_downscale_unchecked_u8_to_u0(#[case] s: &'static str, #[case] expected: &str) {
        rescale_unchecked::<U8, U0>(s, expected);
    }

    #[rstest]
    #[case("50")]
    #[case("12345")]
    fn should_not_rescale_with_same_base_unchecked(#[case] s: &str) {
        let d = DecimalU64::<U2>::from_str(s).unwrap();
        let res: DecimalU64<U2> = unsafe { d.rescale_unchecked() };
        assert_eq!(res.to_string(), d.to_string());
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
        assert_eq!(d.to_string().trim_end_matches('0'), down.to_string().trim_end_matches('0'));
    }

    #[rstest]
    #[case("123.45678900", "123.45")] // U8 -> U2: truncate to 2 decimals
    #[case("500.12345600", "500.12")] // U8 -> U2: truncate to 2 decimals
    #[case("0.99999900", "0.99")] // U8 -> U2: truncate to 2 decimals
    fn should_truncate_downscale_unchecked(#[case] s: &str, #[case] expected: &str) {
        let d = DecimalU64::<U8>::from_str(s).unwrap();
        let res: DecimalU64<U2> = unsafe { d.rescale_unchecked() };
        assert_eq!(res.to_string(), expected);
    }

    #[test]
    fn should_rescale_zero_all_scales() {
        let d = DecimalU64::<U0>::from_str("0").unwrap();
        let up_checked: DecimalU64<U8> = d.rescale().unwrap();
        let up_unchecked: DecimalU64<U8> = unsafe { d.rescale_unchecked() };
        let down_checked: DecimalU64<U0> = up_checked.rescale().unwrap();
        let down_unchecked: DecimalU64<U0> = unsafe { up_unchecked.rescale_unchecked() };

        assert_eq!(up_checked.to_string(), "0.00000000");
        assert_eq!(up_unchecked.to_string(), "0.00000000");
        assert_eq!(down_checked.to_string(), "0");
        assert_eq!(down_unchecked.to_string(), "0");
    }

    #[test]
    fn should_error_on_precision_loss() {
        let d = DecimalU64::<U4>::from_str("101.2038").unwrap(); // 4 decimal places
        let result = d.rescale::<U2>(); // Downscale to 2 decimals

        assert!(result.is_err());
        match result {
            Err(Error::PrecisionLoss(msg)) => {
                assert!(msg.contains("Truncated") || msg.contains("precision"));
            }
            _ => panic!("Expected PrecisionLoss error"),
        }
    }

    #[test]
    fn should_error_on_overflow() {
        // Try to upscale MAX value at U0 to U1 (would multiply by 10, causing overflow)
        let d = DecimalU64::<U0>::MAX;
        let result: Result<DecimalU64<U1>, Error> = d.rescale();

        assert!(result.is_err());
        match result {
            Err(Error::Overflow(_)) => {}
            _ => panic!("Expected Overflow error"),
        }
    }
}
