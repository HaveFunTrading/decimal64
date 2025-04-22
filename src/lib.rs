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
}

gen_scale!(U0, 0);
gen_scale!(U1, 1);
gen_scale!(U2, 2);
gen_scale!(U3, 3);
gen_scale!(U4, 4);
gen_scale!(U5, 5);
gen_scale!(U6, 6);
gen_scale!(U7, 7);
gen_scale!(U8, 8);

const SCALE_FACTORS: [u64; 9] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000];

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DecimalU64<S> {
    pub unscaled: u64,
    phantom: PhantomData<S>,
}

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

        Ok(Self {
            unscaled,
            phantom: PhantomData,
        })
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
    pub const ZERO: Self = DecimalU64::from_raw(0);
    pub const ONE: Self = DecimalU64::from_raw(S::SCALE_FACTOR);
    pub const TWO: Self = DecimalU64::from_raw(2 * S::SCALE_FACTOR);
    pub const THREE: Self = DecimalU64::from_raw(3 * S::SCALE_FACTOR);
    pub const FOUR: Self = DecimalU64::from_raw(4 * S::SCALE_FACTOR);
    pub const FIVE: Self = DecimalU64::from_raw(5 * S::SCALE_FACTOR);
    pub const SIX: Self = DecimalU64::from_raw(6 * S::SCALE_FACTOR);
    pub const SEVEN: Self = DecimalU64::from_raw(7 * S::SCALE_FACTOR);
    pub const EIGHT: Self = DecimalU64::from_raw(8 * S::SCALE_FACTOR);
    pub const NINE: Self = DecimalU64::from_raw(9 * S::SCALE_FACTOR);
    pub const TEN: Self = DecimalU64::from_raw(10 * S::SCALE_FACTOR);
    pub const MAX: Self = DecimalU64::from_raw(u64::MAX);

    #[inline]
    pub const fn from_raw(unscaled: u64) -> Self {
        Self {
            unscaled,
            phantom: PhantomData,
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
        let integer_part = self.unscaled / S::SCALE_FACTOR;
        let fractional_part = self.unscaled % S::SCALE_FACTOR;
        (integer_part, fractional_part)
    }

    #[inline]
    pub fn write_to(&self, buffer: &mut [u8]) -> usize {
        // Compute the scale factor: 10^PRECISION.
        let (int_part, frac_part) = self.split();
        let mut pos = 0;

        // Write the integer part.
        if int_part == 0 {
            buffer[pos] = b'0';
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
                buffer[idx] = b'0' + (tmp % 10) as u8;
                tmp /= 10;
            }
        }

        // If there is a fractional part, write the decimal point and fractional digits.
        if S::SCALE > 0 {
            buffer[pos] = b'.';
            pos += 1;
            // Start with the highest power of 10 for the fractional part.
            let mut divisor = 10u64.pow((S::SCALE - 1) as u32);
            let mut frac = frac_part;
            for _ in 0..S::SCALE {
                let digit = frac / divisor;
                buffer[pos] = b'0' + (digit as u8);
                pos += 1;
                frac %= divisor;
                divisor /= 10;
            }
        }

        pos
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
        assert_eq!(18446744073709551615, DecimalU64::<U0>::try_from("18446744073709551615".as_bytes())?.unscaled);
        assert_eq!(18446744073709551615, DecimalU64::<U8>::try_from("184467440737.09551615".as_bytes())?.unscaled);
        assert_eq!(12345000000, DecimalU64::<U8>::try_from("123.45000000".as_bytes())?.unscaled);
        assert_eq!(12300000000, DecimalU64::<U8>::try_from("123".as_bytes())?.unscaled);
        assert_eq!(12300000000, DecimalU64::<U8>::try_from("123.".as_bytes())?.unscaled);
        assert_eq!(12300000000, DecimalU64::<U8>::try_from("123.0".as_bytes())?.unscaled);
        assert_eq!(18446744073709551615, DecimalU64::<U8>::try_from("184467440737.09551615".as_bytes())?.unscaled);
        assert_eq!(0, DecimalU64::<U8>::try_from("0.0".as_bytes())?.unscaled);
        assert_eq!(0, DecimalU64::<U8>::try_from("0".as_bytes())?.unscaled);
        Ok(())
    }

    #[test]
    fn should_use_target_scale() -> anyhow::Result<()> {
        assert_eq!(12345600000, DecimalU64::<U8>::try_from("123.456".as_bytes())?.unscaled);
        assert_eq!(1234560000, DecimalU64::<U7>::try_from("123.456".as_bytes())?.unscaled);
        assert_eq!(123456000, DecimalU64::<U6>::try_from("123.456".as_bytes())?.unscaled);
        assert_eq!(12345600, DecimalU64::<U5>::try_from("123.456".as_bytes())?.unscaled);
        assert_eq!(1234560, DecimalU64::<U4>::try_from("123.456".as_bytes())?.unscaled);
        assert_eq!(123456, DecimalU64::<U3>::try_from("123.456".as_bytes())?.unscaled);
        assert!(DecimalU64::<U2>::try_from("123.456".as_bytes()).is_err());
        assert!(DecimalU64::<U1>::try_from("123.456".as_bytes()).is_err());
        assert!(DecimalU64::<U0>::try_from("123.456".as_bytes()).is_err());
        Ok(())
    }

    #[test]
    fn should_split() -> anyhow::Result<()> {
        assert_eq!((123, 45000000), DecimalU64::<U8>::try_from("123.45000000".as_bytes())?.split());
        assert_eq!((0, 45000000), DecimalU64::<U8>::try_from("0.45000000".as_bytes())?.split());
        assert_eq!((0, 0), DecimalU64::<U8>::try_from("0.0".as_bytes())?.split());
        assert_eq!((123, 45000001), DecimalU64::<U8>::try_from("123.45000001".as_bytes())?.split());
        assert_eq!((123, 45100000), DecimalU64::<U8>::try_from("123.451".as_bytes())?.split());
        Ok(())
    }

    #[test]
    fn should_compare_for_eq() -> anyhow::Result<()> {
        let one = DecimalU64::<U8>::try_from("123.45000000".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("123.45000000".as_bytes())?;
        assert_eq!(one, two);
        let one = DecimalU64::<U8>::try_from("123.45000000".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("123.45000001".as_bytes())?;
        assert_ne!(one, two);
        let one = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        assert_eq!(one, two);
        Ok(())
    }

    #[test]
    fn should_compare_for_ord() -> anyhow::Result<()> {
        let one = DecimalU64::<U8>::try_from("123.45000001".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("123.45000000".as_bytes())?;
        assert!(one > two);
        let one = DecimalU64::<U8>::try_from("123.45000000".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("123.45000001".as_bytes())?;
        assert!(one < two);
        let one = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        assert!(one >= two);
        let one = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        let two = DecimalU64::<U8>::try_from("0.0".as_bytes())?;
        assert!(one <= two);
        Ok(())
    }

    #[test]
    fn should_err_if_number_too_large() {
        let err = DecimalU64::<U8>::try_from("184467440737.09551616".as_bytes());
        assert!(err.is_err());
        if let Err(err) = err {
            assert_eq!("overflow: 184467440737.09551616", err.to_string());
        }
    }

    #[test]
    fn should_create_from_str() {
        assert_eq!(12345000001, DecimalU64::<U8>::from_str("123.45000001").unwrap().unscaled);
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
        assert_eq!("0.00000123", DecimalU64::<U8>::from_raw(123).to_string());
        assert_eq!("0.0000123", DecimalU64::<U7>::from_raw(123).to_string());
        assert_eq!("123", DecimalU64::<U0>::from_raw(123).to_string());
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
}
