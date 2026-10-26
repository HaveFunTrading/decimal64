use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use decimal64::DecimalU64;
            use decimal64::round::{Ceil, Floor, HalfUp};
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn built_in_policies_match_integer_oracles(raw in any::<u64>(), tick in 1_u64..=u64::MAX) {
                    let value = Decimal::new(raw);
                    let tick_size = Decimal::new(tick);
                    let quotient = raw / tick;
                    let remainder = raw % tick;

                    let floor = quotient * tick;
                    let actual_floor = value.round::<Floor>(tick_size);
                    prop_assert_eq!(actual_floor, Decimal::new(floor));
                    prop_assert!(actual_floor.0 <= raw);
                    prop_assert_eq!(actual_floor.0 % tick, 0);

                    let ceil = if remainder == 0 {
                        raw as u128
                    } else {
                        (quotient as u128 + 1) * tick as u128
                    };
                    if ceil <= u64::MAX as u128 {
                        let actual_ceil = value.round::<Ceil>(tick_size);
                        prop_assert_eq!(actual_ceil, Decimal::new(ceil as u64));
                        prop_assert!(actual_ceil.0 >= raw);
                        prop_assert_eq!(actual_ceil.0 % tick, 0);
                    }

                    let half_up = if remainder >= tick - tick / 2 {
                        (quotient as u128 + 1) * tick as u128
                    } else {
                        floor as u128
                    };
                    if half_up <= u64::MAX as u128 {
                        let actual_half_up = value.round::<HalfUp>(tick_size);
                        prop_assert_eq!(actual_half_up, Decimal::new(half_up as u64));
                        prop_assert_eq!(actual_half_up.0 % tick, 0);
                    }
                }
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
                use decimal64::DecimalI64;
                use decimal64::round::{Ceil, Floor, HalfUp};
                use proptest::prelude::*;

                type Decimal = DecimalI64<$scale>;

                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(512))]

                    #[test]
                    fn built_in_policies_match_integer_oracles(raw in any::<i64>(), tick in 1_i64..=i64::MAX) {
                        let value = Decimal::new(raw);
                        let tick_size = Decimal::new(tick);
                        let raw = raw as i128;
                        let tick = tick as i128;

                        let floor = raw.div_euclid(tick) * tick;
                        if floor >= i64::MIN as i128 {
                            let actual = value.round::<Floor>(tick_size);
                            prop_assert_eq!(actual, Decimal::new(floor as i64));
                            prop_assert!(actual.0 as i128 <= raw);
                            prop_assert_eq!(actual.0 % tick as i64, 0);
                        }

                        let ceil = if raw.rem_euclid(tick) == 0 {
                            raw
                        } else {
                            (raw.div_euclid(tick) + 1) * tick
                        };
                        if ceil <= i64::MAX as i128 {
                            let actual = value.round::<Ceil>(tick_size);
                            prop_assert_eq!(actual, Decimal::new(ceil as i64));
                            prop_assert!(actual.0 as i128 >= raw);
                            prop_assert_eq!(actual.0 % tick as i64, 0);
                        }

                        let magnitude = raw.unsigned_abs();
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
                            prop_assert_eq!(actual, Decimal::new(expected));
                            prop_assert_eq!(actual.0 % tick as i64, 0);
                        }
                    }
                }
            }
        };
    }

    for_each_scale!(suite);
}
