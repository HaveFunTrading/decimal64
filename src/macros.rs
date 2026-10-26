// Shared implementation generators for the signed and unsigned decimal types.
// Keep representation-specific arithmetic and sign handling in their owning
// modules; these macros are only for contracts that must remain identical.

// Representation and constants.
macro_rules! impl_decimal_core {
    ($decimal:ident, $raw:ty, $min:expr, $max:expr) => {
        impl<S: $crate::ScaleMetrics> Default for $decimal<S> {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Creates a decimal from its raw unscaled representation.
            ///
            /// The represented value is `unscaled / S::SCALE_FACTOR`. No
            /// parsing, rounding, or validation is performed.
            #[inline]
            pub const fn new(unscaled: $raw) -> Self {
                Self(unscaled, std::marker::PhantomData)
            }

            /// Returns `true` if this decimal is zero.
            #[must_use]
            #[inline]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }

            /// The value zero.
            pub const ZERO: Self = Self::new(0);
            /// The smallest representable raw value at this scale.
            pub const MIN: Self = Self::new($min);
            /// The value one.
            pub const ONE: Self = Self::new(S::SCALE_FACTOR as $raw);
            /// The value two.
            pub const TWO: Self = Self::new(2 * S::SCALE_FACTOR as $raw);
            /// The value three.
            pub const THREE: Self = Self::new(3 * S::SCALE_FACTOR as $raw);
            /// The value four.
            pub const FOUR: Self = Self::new(4 * S::SCALE_FACTOR as $raw);
            /// The value five.
            pub const FIVE: Self = Self::new(5 * S::SCALE_FACTOR as $raw);
            /// The value six.
            pub const SIX: Self = Self::new(6 * S::SCALE_FACTOR as $raw);
            /// The value seven.
            pub const SEVEN: Self = Self::new(7 * S::SCALE_FACTOR as $raw);
            /// The value eight.
            pub const EIGHT: Self = Self::new(8 * S::SCALE_FACTOR as $raw);
            /// The value nine.
            pub const NINE: Self = Self::new(9 * S::SCALE_FACTOR as $raw);
            /// The value ten.
            pub const TEN: Self = Self::new(10 * S::SCALE_FACTOR as $raw);
            /// The largest representable raw value at this scale.
            pub const MAX: Self = Self::new($max);
        }
    };
}

// Fixed-point arithmetic method families.
macro_rules! impl_decimal_arithmetic {
    ($decimal:ident, $raw:ty) => {
        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Checked fixed-point addition.
            ///
            /// Returns `None` if the result cannot be represented by `Self`.
            #[must_use]
            #[inline]
            pub const fn checked_add(self, rhs: Self) -> Option<Self> {
                match self.0.checked_add(rhs.0) {
                    Some(value) => Some(Self::new(value)),
                    None => None,
                }
            }

            /// Checked fixed-point subtraction.
            ///
            /// Returns `None` if the result cannot be represented by `Self`.
            #[must_use]
            #[inline]
            pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
                match self.0.checked_sub(rhs.0) {
                    Some(value) => Some(Self::new(value)),
                    None => None,
                }
            }

            /// Checked fixed-point multiplication, truncated toward zero.
            ///
            /// Returns `None` if the scaled result cannot be represented by
            /// `Self`.
            #[must_use]
            #[inline]
            pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
                Self::from_wide(self.wide_mul(rhs))
            }

            /// Checked fixed-point division, truncated toward zero.
            ///
            /// Returns `None` for a zero divisor or an unrepresentable result.
            #[must_use]
            #[inline]
            pub const fn checked_div(self, rhs: Self) -> Option<Self> {
                if rhs.0 == 0 {
                    None
                } else {
                    Self::from_wide(self.wide_div(rhs))
                }
            }

            /// Saturating fixed-point addition.
            ///
            /// Clamps the result to the representable range of `Self`.
            #[must_use]
            #[inline]
            pub const fn saturating_add(self, rhs: Self) -> Self {
                Self::new(self.0.saturating_add(rhs.0))
            }

            /// Saturating fixed-point subtraction.
            ///
            /// Clamps the result to the representable range of `Self`.
            #[must_use]
            #[inline]
            pub const fn saturating_sub(self, rhs: Self) -> Self {
                Self::new(self.0.saturating_sub(rhs.0))
            }

            /// Saturating fixed-point multiplication, truncated toward zero.
            ///
            /// Clamps the scaled result to the representable range of `Self`.
            #[must_use]
            #[inline]
            pub const fn saturating_mul(self, rhs: Self) -> Self {
                Self::saturating_from_wide(self.wide_mul(rhs))
            }

            /// Saturating fixed-point division, truncated toward zero.
            ///
            /// Clamps the scaled result to the representable range of `Self`.
            ///
            /// # Panics
            ///
            /// Panics if `rhs` is zero.
            #[must_use]
            #[inline]
            pub const fn saturating_div(self, rhs: Self) -> Self {
                if rhs.0 == 0 {
                    panic!("attempt to divide by zero");
                }
                Self::saturating_from_wide(self.wide_div(rhs))
            }

            /// Wrapping fixed-point addition.
            ///
            /// Wraps the raw result as the backing integer type.
            #[must_use]
            #[inline]
            pub const fn wrapping_add(self, rhs: Self) -> Self {
                Self::new(self.0.wrapping_add(rhs.0))
            }

            /// Wrapping fixed-point subtraction.
            ///
            /// Wraps the raw result as the backing integer type.
            #[must_use]
            #[inline]
            pub const fn wrapping_sub(self, rhs: Self) -> Self {
                Self::new(self.0.wrapping_sub(rhs.0))
            }

            /// Wrapping fixed-point multiplication, truncated toward zero.
            ///
            /// Wraps the scaled result as the backing integer type.
            #[must_use]
            #[inline]
            pub const fn wrapping_mul(self, rhs: Self) -> Self {
                Self::new(self.wide_mul(rhs) as $raw)
            }

            /// Wrapping fixed-point division, truncated toward zero.
            ///
            /// Wraps the scaled result as the backing integer type.
            ///
            /// # Panics
            ///
            /// Panics if `rhs` is zero.
            #[must_use]
            #[inline]
            pub const fn wrapping_div(self, rhs: Self) -> Self {
                if rhs.0 == 0 {
                    panic!("attempt to divide by zero");
                }
                Self::new(self.wide_div(rhs) as $raw)
            }

            /// Overflowing fixed-point addition.
            ///
            /// Returns the wrapped result and whether overflow occurred.
            #[must_use]
            #[inline]
            pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                let (value, overflowed) = self.0.overflowing_add(rhs.0);
                (Self::new(value), overflowed)
            }

            /// Overflowing fixed-point subtraction.
            ///
            /// Returns the wrapped result and whether overflow occurred.
            #[must_use]
            #[inline]
            pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                let (value, overflowed) = self.0.overflowing_sub(rhs.0);
                (Self::new(value), overflowed)
            }

            /// Overflowing fixed-point multiplication, truncated toward zero.
            ///
            /// Returns the wrapped result and whether overflow occurred.
            #[must_use]
            #[inline]
            pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
                let value = self.wide_mul(rhs);
                let overflowed = match Self::from_wide(value) {
                    Some(_) => false,
                    None => true,
                };
                (Self::new(value as $raw), overflowed)
            }

            /// Overflowing fixed-point division, truncated toward zero.
            ///
            /// Returns the wrapped result and whether overflow occurred.
            ///
            /// # Panics
            ///
            /// Panics if `rhs` is zero.
            #[must_use]
            #[inline]
            pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
                if rhs.0 == 0 {
                    panic!("attempt to divide by zero");
                }
                let value = self.wide_div(rhs);
                let overflowed = match Self::from_wide(value) {
                    Some(_) => false,
                    None => true,
                };
                (Self::new(value as $raw), overflowed)
            }
        }
    };
}

// Standard arithmetic traits.
macro_rules! impl_binary_operator {
    ($decimal:ident, $trait:ident, $method:ident, $checked:ident, $message:literal) => {
        impl<S: $crate::ScaleMetrics> std::ops::$trait for $decimal<S> {
            type Output = Self;

            #[inline]
            fn $method(self, rhs: Self) -> Self::Output {
                self.$checked(rhs).expect($message)
            }
        }
    };
}

macro_rules! impl_assign_operator {
    ($decimal:ident, $trait:ident, $method:ident, $operator:tt) => {
        impl<S: $crate::ScaleMetrics> std::ops::$trait for $decimal<S> {
            #[inline]
            fn $method(&mut self, rhs: Self) {
                *self = *self $operator rhs;
            }
        }

        impl<S: $crate::ScaleMetrics> std::ops::$trait<&$decimal<S>> for $decimal<S> {
            #[inline]
            fn $method(&mut self, rhs: &Self) {
                *self = *self $operator *rhs;
            }
        }
    };
}

macro_rules! impl_decimal_operators {
    ($decimal:ident) => {
        impl_binary_operator!($decimal, Add, add, checked_add, "attempt to add with overflow");
        impl_binary_operator!($decimal, Sub, sub, checked_sub, "attempt to subtract with overflow");
        impl_binary_operator!($decimal, Mul, mul, checked_mul, "attempt to multiply with overflow");

        impl<S: $crate::ScaleMetrics> std::ops::Div for $decimal<S> {
            type Output = Self;

            #[inline]
            fn div(self, rhs: Self) -> Self::Output {
                if rhs.0 == 0 {
                    panic!("attempt to divide by zero");
                }
                self.checked_div(rhs).expect("attempt to divide with overflow")
            }
        }

        impl_assign_operator!($decimal, AddAssign, add_assign, +);
        impl_assign_operator!($decimal, SubAssign, sub_assign, -);
        impl_assign_operator!($decimal, MulAssign, mul_assign, *);
        impl_assign_operator!($decimal, DivAssign, div_assign, /);

        impl<S: $crate::ScaleMetrics> std::iter::Sum for $decimal<S> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, std::ops::Add::add)
            }
        }

        impl<'a, S: $crate::ScaleMetrics> std::iter::Sum<&'a Self> for $decimal<S> {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.copied().sum()
            }
        }

        impl<S: $crate::ScaleMetrics> std::iter::Product for $decimal<S> {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ONE, std::ops::Mul::mul)
            }
        }

        impl<'a, S: $crate::ScaleMetrics> std::iter::Product<&'a Self> for $decimal<S> {
            fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.copied().product()
            }
        }
    };
}

// String parsing traits and convenience methods.
macro_rules! impl_decimal_string_parsing {
    ($decimal:ident) => {
        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Parses an exactly representable decimal from a UTF-8 string
            /// slice.
            ///
            /// Fractional digits beyond the target scale are accepted only
            /// when they are zero. This method never rounds.
            #[inline]
            pub const fn from_str(value: &str) -> Result<Self, $crate::Error> {
                Self::from_slice(value.as_bytes())
            }
        }

        impl<S: $crate::ScaleMetrics> std::str::FromStr for $decimal<S> {
            type Err = $crate::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::from_slice(value.as_bytes())
            }
        }
    };
}

// Buffer writers and Display.
macro_rules! impl_decimal_formatting {
    ($decimal:ident) => {
        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Writes this decimal into `buffer`, including trailing
            /// fractional zeros, and returns the number of bytes written.
            ///
            /// # Panics
            ///
            /// Panics if `buffer` is too small.
            #[must_use]
            #[inline]
            pub fn write_to(&self, buffer: &mut [u8]) -> usize {
                match self.try_write_to(buffer) {
                    Ok(len) => len,
                    Err(error) => panic!("{error}"),
                }
            }

            /// Tries to write this decimal into `buffer`, including trailing
            /// fractional zeros.
            ///
            /// # Errors
            ///
            /// Returns [`crate::BufferTooSmall`] without modifying `buffer`
            /// if it cannot hold the formatted value.
            #[inline]
            pub fn try_write_to(&self, buffer: &mut [u8]) -> Result<usize, $crate::BufferTooSmall> {
                self.try_write_impl(buffer, false)
            }

            /// Writes this decimal into `buffer` without trailing fractional
            /// zeros and returns the number of bytes written.
            ///
            /// # Panics
            ///
            /// Panics if `buffer` is too small.
            #[must_use]
            #[inline]
            pub fn write_to_trimmed(&self, buffer: &mut [u8]) -> usize {
                match self.try_write_to_trimmed(buffer) {
                    Ok(len) => len,
                    Err(error) => panic!("{error}"),
                }
            }

            /// Tries to write this decimal into `buffer` without trailing
            /// fractional zeros.
            ///
            /// # Errors
            ///
            /// Returns [`crate::BufferTooSmall`] without modifying `buffer`
            /// if it cannot hold the formatted value.
            #[inline]
            pub fn try_write_to_trimmed(&self, buffer: &mut [u8]) -> Result<usize, $crate::BufferTooSmall> {
                self.try_write_impl(buffer, true)
            }
        }

        impl<S: $crate::ScaleMetrics> std::fmt::Display for $decimal<S> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut buffer = [0u8; 21];
                let len = self.write_to(&mut buffer);
                // SAFETY: decimal formatting emits ASCII only.
                let value = unsafe { std::str::from_utf8_unchecked(&buffer[..len]) };
                formatter.write_str(value)
            }
        }
    };
}

// Conversions whose behavior is identical for either backing integer.
macro_rules! impl_decimal_float_conversions {
    ($decimal:ident) => {
        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Converts this decimal to `f64`.
            #[inline]
            pub const fn to_f64(self) -> f64 {
                self.0 as f64 / S::SCALE_FACTOR as f64
            }
        }

        impl<S: $crate::ScaleMetrics> TryFrom<f64> for $decimal<S> {
            type Error = $crate::Error;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                Self::from_f64(value)
            }
        }

        impl<S: $crate::ScaleMetrics> From<$decimal<S>> for f64 {
            fn from(value: $decimal<S>) -> Self {
                value.to_f64()
            }
        }
    };
}

// The inherent entry point shared by all rounding policies.
macro_rules! impl_decimal_round {
    ($decimal:ident) => {
        impl<S: $crate::ScaleMetrics> $decimal<S> {
            /// Rounds to `tick_size` using `R`.
            ///
            /// # Panics
            ///
            /// Panics if `tick_size` is zero, if it is negative for a signed
            /// decimal, or if the rounded result cannot be represented by
            /// `Self`.
            #[must_use]
            pub fn round<R: $crate::round::RoundingPolicy<Self>>(self, tick_size: Self) -> Self {
                R::round(self, tick_size)
            }
        }
    };
}

// String-only serde representation.
#[cfg(feature = "serde")]
macro_rules! impl_decimal_serde {
    ($decimal:ident, $visitor:ident, $expecting:literal) => {
        impl<S: $crate::ScaleMetrics> serde::Serialize for $decimal<S> {
            fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
            where
                Ser: serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        struct $visitor<S> {
            marker: std::marker::PhantomData<S>,
        }

        impl<'de, S: $crate::ScaleMetrics> serde::de::Visitor<'de> for $visitor<S> {
            type Value = $decimal<S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str($expecting)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                $decimal::<S>::from_str(value).map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }
        }

        impl<'de, S: $crate::ScaleMetrics> serde::Deserialize<'de> for $decimal<S> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str($visitor {
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}
