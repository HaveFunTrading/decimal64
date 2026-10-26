use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use crate::common::{positive_u64_strategy, unsigned_boundary_values, unsigned_raw_strategy};
            use decimal64::DecimalU64;
            use decimal64::round::{Ceil, Floor, HalfUp};
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            fn assert_rounding(raw: u64, tick: u64) {
                assert_ne!(tick, 0);
                let value = Decimal::new(raw);
                let tick_size = Decimal::new(tick);
                let quotient = raw / tick;
                let remainder = raw % tick;

                let floor = quotient * tick;
                let actual_floor = value.round::<Floor>(tick_size);
                assert_eq!(actual_floor, Decimal::new(floor));
                assert!(actual_floor.0 <= raw);
                assert_eq!(actual_floor.0 % tick, 0);

                let ceil = if remainder == 0 {
                    raw as u128
                } else {
                    (quotient as u128 + 1) * tick as u128
                };
                if ceil <= u64::MAX as u128 {
                    let actual = value.round::<Ceil>(tick_size);
                    assert_eq!(actual, Decimal::new(ceil as u64));
                    assert!(actual.0 >= raw);
                    assert_eq!(actual.0 % tick, 0);
                } else {
                    assert!(std::panic::catch_unwind(|| value.round::<Ceil>(tick_size)).is_err());
                }

                let half_up = if remainder >= tick - tick / 2 {
                    (quotient as u128 + 1) * tick as u128
                } else {
                    floor as u128
                };
                if half_up <= u64::MAX as u128 {
                    let actual = value.round::<HalfUp>(tick_size);
                    assert_eq!(actual, Decimal::new(half_up as u64));
                    assert_eq!(actual.0 % tick, 0);
                } else {
                    assert!(std::panic::catch_unwind(|| value.round::<HalfUp>(tick_size)).is_err());
                }
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn built_in_policies_match_integer_oracles(
                    raw in unsigned_raw_strategy($factor),
                    tick in positive_u64_strategy($factor),
                ) {
                    assert_rounding(raw, tick);
                }
            }

            #[test]
            fn deterministic_boundaries_and_midpoint_neighbours() {
                let mut ticks = vec![1, 2, 3, 4, 5, 10, $factor, u64::MAX];
                ticks.sort_unstable();
                ticks.dedup();

                for raw in unsigned_boundary_values($factor) {
                    for &tick in &ticks {
                        assert_rounding(raw, tick);
                    }
                }

                for tick in ticks {
                    let midpoint = tick - tick / 2;
                    let remainders = [
                        midpoint.saturating_sub(1),
                        midpoint,
                        midpoint.saturating_add(1).min(tick - 1),
                    ];
                    for quotient in [0, 1, u64::MAX / tick] {
                        let base = quotient as u128 * tick as u128;
                        for remainder in remainders {
                            let raw = base + remainder as u128;
                            if raw <= u64::MAX as u128 {
                                assert_rounding(raw as u64, tick);
                            }
                        }
                    }
                }
            }

            #[test]
            fn every_policy_rejects_zero_tick() {
                let value = Decimal::ONE;
                let tick = Decimal::ZERO;
                assert!(std::panic::catch_unwind(|| value.round::<HalfUp>(tick)).is_err());
                assert!(std::panic::catch_unwind(|| value.round::<Floor>(tick)).is_err());
                assert!(std::panic::catch_unwind(|| value.round::<Ceil>(tick)).is_err());
            }
        }
    };
}

for_each_scale!(suite);

mod signed {
    use crate::common::for_each_scale;

    macro_rules! suite {
        ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
            mod $module {
                use crate::common::{positive_i64_strategy, signed_boundary_values, signed_raw_strategy};
                use decimal64::DecimalI64;
                use decimal64::round::{Ceil, Floor, HalfUp};
                use proptest::prelude::*;

                type Decimal = DecimalI64<$scale>;

                fn assert_rounding(raw: i64, tick: i64) {
                    assert!(tick > 0);
                    let value = Decimal::new(raw);
                    let tick_size = Decimal::new(tick);
                    let raw_wide = raw as i128;
                    let tick_wide = tick as i128;

                    let floor = raw_wide.div_euclid(tick_wide) * tick_wide;
                    if floor >= i64::MIN as i128 {
                        let actual = value.round::<Floor>(tick_size);
                        assert_eq!(actual, Decimal::new(floor as i64));
                        assert!(actual.0 as i128 <= raw_wide);
                        assert_eq!(actual.0 % tick, 0);
                    } else {
                        assert!(std::panic::catch_unwind(|| value.round::<Floor>(tick_size)).is_err());
                    }

                    let ceil = if raw_wide.rem_euclid(tick_wide) == 0 {
                        raw_wide
                    } else {
                        (raw_wide.div_euclid(tick_wide) + 1) * tick_wide
                    };
                    if ceil <= i64::MAX as i128 {
                        let actual = value.round::<Ceil>(tick_size);
                        assert_eq!(actual, Decimal::new(ceil as i64));
                        assert!(actual.0 as i128 >= raw_wide);
                        assert_eq!(actual.0 % tick, 0);
                    } else {
                        assert!(std::panic::catch_unwind(|| value.round::<Ceil>(tick_size)).is_err());
                    }

                    let magnitude = raw_wide.unsigned_abs();
                    let tick_magnitude = tick as u128;
                    let quotient = magnitude / tick_magnitude;
                    let remainder = magnitude % tick_magnitude;
                    let rounded_magnitude = if remainder >= tick_magnitude - tick_magnitude / 2 {
                        (quotient + 1) * tick_magnitude
                    } else {
                        quotient * tick_magnitude
                    };
                    let limit = if raw < 0 { 1_u128 << 63 } else { i64::MAX as u128 };
                    if rounded_magnitude <= limit {
                        let expected = if raw < 0 {
                            -(rounded_magnitude as i128) as i64
                        } else {
                            rounded_magnitude as i64
                        };
                        let actual = value.round::<HalfUp>(tick_size);
                        assert_eq!(actual, Decimal::new(expected));
                        assert_eq!(actual.0 % tick, 0);
                    } else {
                        assert!(std::panic::catch_unwind(|| value.round::<HalfUp>(tick_size)).is_err());
                    }
                }

                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(512))]

                    #[test]
                    fn built_in_policies_match_integer_oracles(
                        raw in signed_raw_strategy($factor),
                        tick in positive_i64_strategy($factor),
                    ) {
                        assert_rounding(raw, tick);
                    }
                }

                #[test]
                fn deterministic_boundaries_midpoints_and_minimum() {
                    let mut ticks = vec![1, 2, 3, 4, 5, 10, $factor as i64, i64::MAX];
                    ticks.sort_unstable();
                    ticks.dedup();

                    for raw in signed_boundary_values($factor) {
                        for &tick in &ticks {
                            assert_rounding(raw, tick);
                        }
                    }

                    for tick in ticks {
                        let midpoint = tick - tick / 2;
                        let remainders = [
                            midpoint.saturating_sub(1),
                            midpoint,
                            midpoint.saturating_add(1).min(tick - 1),
                        ];
                        for quotient in [0_i128, 1, i64::MAX as i128 / tick as i128] {
                            let base = quotient * tick as i128;
                            for remainder in remainders {
                                let magnitude = base + remainder as i128;
                                for raw in [magnitude, -magnitude] {
                                    if (i64::MIN as i128..=i64::MAX as i128).contains(&raw) {
                                        assert_rounding(raw as i64, tick);
                                    }
                                }
                            }
                        }
                    }
                }

                #[test]
                fn every_policy_rejects_non_positive_ticks() {
                    let value = Decimal::ONE;
                    for tick in [Decimal::ZERO, Decimal::new(-1), Decimal::MIN] {
                        assert!(std::panic::catch_unwind(|| value.round::<HalfUp>(tick)).is_err());
                        assert!(std::panic::catch_unwind(|| value.round::<Floor>(tick)).is_err());
                        assert!(std::panic::catch_unwind(|| value.round::<Ceil>(tick)).is_err());
                    }
                }
            }
        };
    }

    for_each_scale!(suite);
}
