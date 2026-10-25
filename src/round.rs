use crate::{DecimalU64, ScaleMetrics};

pub trait RoundingPolicy {
    /// Rounds `value` to `tick_size`.
    fn round<S: ScaleMetrics>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S>;
}

///  Round‑half‑up (“.5 → up”), e.g. 0.125 at tick 0.01 → 0.13.
pub struct HalfUp;

impl RoundingPolicy for HalfUp {
    #[inline]
    fn round<S: ScaleMetrics>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        let tick = tick_size.0;
        if tick == 0 {
            panic!("attempt to round with a zero tick size");
        }

        let quotient = value.0 / tick;
        let remainder = value.0 % tick;
        let round_up_at = tick - tick / 2;
        let rounded = if remainder >= round_up_at {
            quotient
                .checked_add(1)
                .and_then(|value| value.checked_mul(tick))
                .expect("attempt to round with overflow")
        } else {
            quotient * tick
        };
        DecimalU64::new(rounded)
    }
}

/// Always down, e.g. 0.129 at tick 0.01 → 0.12.
pub struct Floor;

impl RoundingPolicy for Floor {
    #[inline]
    fn round<S: ScaleMetrics>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        let tick = tick_size.0;
        if tick == 0 {
            panic!("attempt to round with a zero tick size");
        }

        DecimalU64::new((value.0 / tick) * tick)
    }
}

/// Always up (if not exact), e.g. 0.121 at tick 0.01 → 0.13.
pub struct Ceil;

impl RoundingPolicy for Ceil {
    fn round<S: ScaleMetrics>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        let tick = tick_size.0;
        if tick == 0 {
            panic!("attempt to round with a zero tick size");
        }

        let quotient = value.0 / tick;
        let remainder = value.0 % tick;
        let rounded = if remainder == 0 {
            value.0
        } else {
            quotient
                .checked_add(1)
                .and_then(|value| value.checked_mul(tick))
                .expect("attempt to round with overflow")
        };
        DecimalU64::new(rounded)
    }
}

impl<S: ScaleMetrics> DecimalU64<S> {
    /// Rounds to `tick_size` using `R`.
    ///
    /// # Panics
    ///
    /// Panics if `tick_size` is zero or the rounded result cannot be represented by `Self`.
    #[must_use]
    pub fn round<R: RoundingPolicy>(self, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        R::round(self, tick_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::U8;
    use rstest_macros::rstest;

    struct Identity;

    impl RoundingPolicy for Identity {
        fn round<S: ScaleMetrics>(value: DecimalU64<S>, _tick_size: DecimalU64<S>) -> DecimalU64<S> {
            value
        }
    }

    #[test]
    fn should_support_custom_rounding_policies() {
        let value = DecimalU64::<U8>::new(123);
        assert_eq!(value.round::<Identity>(DecimalU64::ZERO), value);
    }

    #[rstest]
    #[case("300.00", "0.1", "300.00000000")]
    #[case("300.02", "0.1", "300.00000000")]
    #[case("300.04", "0.1", "300.00000000")]
    #[case("300.05", "0.1", "300.10000000")]
    #[case("300.06", "0.1", "300.10000000")]
    #[case("0.0643", "0.1", "0.10000000")]
    #[case("0.0543", "0.1", "0.10000000")]
    #[case("0.0443", "0.1", "0.00000000")]
    #[case("1.0443", "0.1", "1.00000000")]
    #[case("1.01", "0.01", "1.01000000")]
    #[case("1.02", "0.01", "1.02000000")]
    #[case("1.0543", "0.01", "1.05000000")]
    #[case("1.0563", "0.01", "1.06000000")]
    #[case("1.0543", "0.05", "1.05000000")]
    #[case("1.0563", "0.05", "1.05000000")]
    #[case("1.0666", "0.05", "1.05000000")]
    #[case("1.075", "0.05", "1.10000000")]
    // Odd unscaled tick: there is no exactly representable halfway value.
    #[case("0.00000002", "0.00000005", "0.00000000")]
    #[case("0.00000003", "0.00000005", "0.00000005")]
    #[case("0.00000007", "0.00000005", "0.00000005")]
    #[case("0.00000008", "0.00000005", "0.00000010")]
    fn should_round_using_round_half_up(#[case] value: &str, #[case] tick_size: &str, #[case] expected: &str) {
        assert_eq!(
            expected,
            DecimalU64::<U8>::from_str(value)
                .unwrap()
                .round::<HalfUp>(DecimalU64::<U8>::from_str(tick_size).unwrap())
                .to_string()
        );
    }

    #[rstest]
    #[case("0.129", "0.01", "0.12000000")]
    #[case("0.12", "0.01", "0.12000000")]
    #[case("300.00", "0.1", "300.00000000")]
    #[case("300.001", "0.1", "300.00000000")]
    #[case("300.971", "0.1", "300.90000000")]
    #[case("300.971", "0.5", "300.50000000")]
    fn should_round_using_round_floor(#[case] value: &str, #[case] tick_size: &str, #[case] expected: &str) {
        assert_eq!(
            expected,
            DecimalU64::<U8>::from_str(value)
                .unwrap()
                .round::<Floor>(DecimalU64::<U8>::from_str(tick_size).unwrap())
                .to_string()
        );
    }

    #[rstest]
    #[case("0.121", "0.01", "0.13000000")]
    #[case("0.12", "0.01", "0.12000000")]
    #[case("300.12345", "0.1", "300.20000000")]
    #[case("300.12345", "0.01", "300.13000000")]
    #[case("300.12345", "0.05", "300.15000000")]
    fn should_round_using_round_ceil(#[case] value: &str, #[case] tick_size: &str, #[case] expected: &str) {
        assert_eq!(
            expected,
            DecimalU64::<U8>::from_str(value)
                .unwrap()
                .round::<Ceil>(DecimalU64::<U8>::from_str(tick_size).unwrap())
                .to_string()
        );
    }

    #[test]
    fn should_round_down_at_the_numeric_bound() {
        let rounded = DecimalU64::<U8>::MAX.round::<Floor>(DecimalU64::new(10));
        assert_eq!(rounded, DecimalU64::new(u64::MAX - 5));
    }

    #[test]
    #[should_panic(expected = "attempt to round with a zero tick size")]
    fn should_panic_when_rounding_with_a_zero_tick() {
        let _ = DecimalU64::<U8>::ONE.round::<Floor>(DecimalU64::ZERO);
    }

    #[test]
    #[should_panic(expected = "attempt to round with overflow")]
    fn should_panic_when_rounding_overflows() {
        let _ = DecimalU64::<U8>::MAX.round::<Ceil>(DecimalU64::new(10));
    }

    #[test]
    #[should_panic(expected = "attempt to round with overflow")]
    fn should_panic_when_half_up_rounding_overflows() {
        let _ = DecimalU64::<U8>::MAX.round::<HalfUp>(DecimalU64::new(10));
    }
}
