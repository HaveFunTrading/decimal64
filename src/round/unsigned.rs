use super::{Ceil, Floor, HalfUp, RoundingPolicy};
use crate::{DecimalU64, ScaleMetrics};

impl<S: ScaleMetrics> RoundingPolicy<DecimalU64<S>> for HalfUp {
    #[inline]
    fn round(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
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

impl<S: ScaleMetrics> RoundingPolicy<DecimalU64<S>> for Floor {
    #[inline]
    fn round(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
        let tick = tick_size.0;
        if tick == 0 {
            panic!("attempt to round with a zero tick size");
        }

        DecimalU64::new((value.0 / tick) * tick)
    }
}

impl<S: ScaleMetrics> RoundingPolicy<DecimalU64<S>> for Ceil {
    fn round(value: DecimalU64<S>, tick_size: DecimalU64<S>) -> DecimalU64<S> {
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
