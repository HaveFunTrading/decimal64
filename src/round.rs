use crate::{DecimalU64, ScaleMetrics};

pub trait RoundingPolicy {
    fn round<S: ScaleMetrics + Copy>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S>;
}

///  Round‑half‑up (“.5 → up”), e.g. 0.125 at tick 0.01 → 0.13.
pub struct HalfUp;

impl RoundingPolicy for HalfUp {
    #[inline]
    fn round<S: ScaleMetrics + Copy>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        let half_tick = tick_size.unscaled / 2 + (tick_size.unscaled % 2);
        DecimalU64::from_raw(((value.unscaled + half_tick) / tick_size.unscaled) * tick_size.unscaled)
    }
}

/// Always down, e.g. 0.129 at tick 0.01 → 0.12.
pub struct Floor;

impl RoundingPolicy for Floor {
    #[inline]
    fn round<S: ScaleMetrics + Copy>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        DecimalU64::from_raw((value.unscaled / tick_size.unscaled) * tick_size.unscaled)
    }
}

/// Always up (if not exact), e.g. 0.121 at tick 0.01 → 0.13.
pub struct Ceil;

impl RoundingPolicy for Ceil {
    #[allow(clippy::manual_div_ceil)]
    fn round<S: ScaleMetrics + Copy>(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        DecimalU64::from_raw(((value.unscaled + tick_size.unscaled - 1) / tick_size.unscaled) * tick_size.unscaled)
    }
}

impl<S: ScaleMetrics + Copy> DecimalU64<S> {
    pub fn round<R: RoundingPolicy>(self, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        R::round(self, tick_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::U8;
    use rstest_macros::rstest;
    use std::str::FromStr;

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
    #[case("1.0543", "0.01", "1.05000000")]
    #[case("1.0563", "0.01", "1.06000000")]
    #[case("1.0543", "0.05", "1.05000000")]
    #[case("1.0563", "0.05", "1.05000000")]
    #[case("1.0666", "0.05", "1.05000000")]
    #[case("1.075", "0.05", "1.10000000")]
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
}
