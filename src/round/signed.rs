use super::{Ceil, Floor, HalfUp, RoundingPolicy};
use crate::{DecimalI64, ScaleMetrics};

impl<S: ScaleMetrics> RoundingPolicy<DecimalI64<S>> for HalfUp {
    #[inline]
    fn round(value: DecimalI64<S>, tick_size: DecimalI64<S>) -> DecimalI64<S> {
        let tick = positive_tick(tick_size.0) as u64;
        let magnitude = value.0.unsigned_abs();
        let quotient = magnitude / tick;
        let remainder = magnitude % tick;
        let round_up_at = tick - tick / 2;
        let rounded = if remainder >= round_up_at {
            (quotient as u128 + 1) * tick as u128
        } else {
            quotient as u128 * tick as u128
        };
        signed_from_magnitude(rounded, value.0 < 0)
    }
}

impl<S: ScaleMetrics> RoundingPolicy<DecimalI64<S>> for Floor {
    #[inline]
    fn round(value: DecimalI64<S>, tick_size: DecimalI64<S>) -> DecimalI64<S> {
        let tick = positive_tick(tick_size.0) as i128;
        let rounded = (value.0 as i128).div_euclid(tick) * tick;
        signed_from_wide(rounded)
    }
}

impl<S: ScaleMetrics> RoundingPolicy<DecimalI64<S>> for Ceil {
    #[inline]
    fn round(value: DecimalI64<S>, tick_size: DecimalI64<S>) -> DecimalI64<S> {
        let tick = positive_tick(tick_size.0) as i128;
        let value = value.0 as i128;
        let quotient = value.div_euclid(tick);
        let rounded = if value.rem_euclid(tick) == 0 {
            value
        } else {
            (quotient + 1) * tick
        };
        signed_from_wide(rounded)
    }
}

#[inline]
fn positive_tick(tick: i64) -> i64 {
    if tick <= 0 {
        panic!("attempt to round with a non-positive tick size");
    }
    tick
}

#[inline]
fn signed_from_wide<S: ScaleMetrics>(value: i128) -> DecimalI64<S> {
    if value < i64::MIN as i128 || value > i64::MAX as i128 {
        panic!("attempt to round with overflow");
    }
    DecimalI64::new(value as i64)
}

#[inline]
fn signed_from_magnitude<S: ScaleMetrics>(magnitude: u128, negative: bool) -> DecimalI64<S> {
    if negative {
        if magnitude > 1u128 << 63 {
            panic!("attempt to round with overflow");
        }
        if magnitude == 1u128 << 63 {
            DecimalI64::MIN
        } else {
            DecimalI64::new(-(magnitude as i64))
        }
    } else {
        if magnitude > i64::MAX as u128 {
            panic!("attempt to round with overflow");
        }
        DecimalI64::new(magnitude as i64)
    }
}
