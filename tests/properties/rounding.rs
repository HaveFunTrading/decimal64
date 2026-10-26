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
