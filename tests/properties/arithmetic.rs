use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use crate::common::reference::{ArithmeticOperation, unsigned_arithmetic_outcome};
            use crate::common::{
                unsigned_boundary_values, unsigned_div_threshold_pairs, unsigned_mul_threshold_pairs,
                unsigned_raw_strategy,
            };
            use decimal64::DecimalU64;
            use num_bigint::BigInt;
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            fn assert_add(lhs: u64, rhs: u64) {
                let lhs_value = Decimal::new(lhs);
                let rhs_value = Decimal::new(rhs);
                let expected = unsigned_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Add).unwrap();
                assert_eq!(
                    lhs_value.checked_add(rhs_value).map(|value| value.0),
                    expected.checked
                );
                assert_eq!(lhs_value.wrapping_add(rhs_value).0, expected.wrapping);
                assert_eq!(
                    lhs_value.overflowing_add(rhs_value),
                    (Decimal::new(expected.wrapping), expected.overflowing)
                );
                assert_eq!(lhs_value.saturating_add(rhs_value).0, expected.saturating);
                if let Some(expected) = expected.checked {
                    assert_eq!((lhs_value + rhs_value).0, expected);
                    let mut assigned = lhs_value;
                    assigned += rhs_value;
                    assert_eq!(assigned.0, expected);
                }
            }

            fn assert_sub(lhs: u64, rhs: u64) {
                let lhs_value = Decimal::new(lhs);
                let rhs_value = Decimal::new(rhs);
                let expected = unsigned_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Sub).unwrap();
                assert_eq!(
                    lhs_value.checked_sub(rhs_value).map(|value| value.0),
                    expected.checked
                );
                assert_eq!(lhs_value.wrapping_sub(rhs_value).0, expected.wrapping);
                assert_eq!(
                    lhs_value.overflowing_sub(rhs_value),
                    (Decimal::new(expected.wrapping), expected.overflowing)
                );
                assert_eq!(lhs_value.saturating_sub(rhs_value).0, expected.saturating);
                if let Some(expected) = expected.checked {
                    assert_eq!((lhs_value - rhs_value).0, expected);
                    let mut assigned = lhs_value;
                    assigned -= rhs_value;
                    assert_eq!(assigned.0, expected);
                }
            }

            fn assert_mul(lhs: u64, rhs: u64) {
                let lhs_value = Decimal::new(lhs);
                let rhs_value = Decimal::new(rhs);
                let expected = unsigned_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Mul).unwrap();
                assert_eq!(
                    lhs_value.checked_mul(rhs_value).map(|value| value.0),
                    expected.checked
                );
                assert_eq!(lhs_value.wrapping_mul(rhs_value).0, expected.wrapping);
                assert_eq!(
                    lhs_value.overflowing_mul(rhs_value),
                    (Decimal::new(expected.wrapping), expected.overflowing)
                );
                assert_eq!(lhs_value.saturating_mul(rhs_value).0, expected.saturating);
                if let Some(expected) = expected.checked {
                    assert_eq!((lhs_value * rhs_value).0, expected);
                    let mut assigned = lhs_value;
                    assigned *= rhs_value;
                    assert_eq!(assigned.0, expected);
                }

                let numerator = BigInt::from(lhs) * BigInt::from(rhs);
                let remainder = numerator - &expected.exact * BigInt::from($factor);
                assert!(remainder >= BigInt::from(0_u8));
                assert!(remainder < BigInt::from($factor));
            }

            fn assert_div(lhs: u64, rhs: u64) {
                let lhs_value = Decimal::new(lhs);
                let rhs_value = Decimal::new(rhs);
                if rhs == 0 {
                    assert_eq!(lhs_value.checked_div(rhs_value), None);
                    return;
                }

                let expected = unsigned_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Div).unwrap();
                assert_eq!(
                    lhs_value.checked_div(rhs_value).map(|value| value.0),
                    expected.checked
                );
                assert_eq!(lhs_value.wrapping_div(rhs_value).0, expected.wrapping);
                assert_eq!(
                    lhs_value.overflowing_div(rhs_value),
                    (Decimal::new(expected.wrapping), expected.overflowing)
                );
                assert_eq!(lhs_value.saturating_div(rhs_value).0, expected.saturating);
                if let Some(expected) = expected.checked {
                    assert_eq!((lhs_value / rhs_value).0, expected);
                    let mut assigned = lhs_value;
                    assigned /= rhs_value;
                    assert_eq!(assigned.0, expected);
                }

                let numerator = BigInt::from(lhs) * BigInt::from($factor);
                let remainder = numerator - &expected.exact * BigInt::from(rhs);
                assert!(remainder >= BigInt::from(0_u8));
                assert!(remainder < BigInt::from(rhs));
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn operation_families_match_bigint_oracles(
                    lhs in unsigned_raw_strategy($factor),
                    rhs in unsigned_raw_strategy($factor),
                ) {
                    assert_add(lhs, rhs);
                    assert_sub(lhs, rhs);
                    assert_mul(lhs, rhs);
                    assert_div(lhs, rhs);
                }

                #[test]
                fn arithmetic_identities_hold(raw in unsigned_raw_strategy($factor)) {
                    let value = Decimal::new(raw);
                    prop_assert_eq!(value.is_zero(), raw == 0);
                    prop_assert_eq!(value.checked_add(Decimal::ZERO), Some(value));
                    prop_assert_eq!(value.checked_sub(Decimal::ZERO), Some(value));
                    prop_assert_eq!(value.checked_mul(Decimal::ONE), Some(value));
                    prop_assert_eq!(value.checked_div(Decimal::ONE), Some(value));
                    prop_assert_eq!(Decimal::ZERO.checked_mul(value), Some(Decimal::ZERO));
                }
            }

            #[test]
            fn deterministic_boundary_matrix_matches_bigint_oracles() {
                let values = unsigned_boundary_values($factor);
                for &lhs in &values {
                    for &rhs in &values {
                        assert_add(lhs, rhs);
                        assert_sub(lhs, rhs);
                        assert_mul(lhs, rhs);
                        assert_div(lhs, rhs);
                    }
                }

                for (lhs, rhs) in unsigned_mul_threshold_pairs($factor) {
                    assert_mul(lhs, rhs);
                }
                for (lhs, rhs) in unsigned_div_threshold_pairs($factor) {
                    assert_div(lhs, rhs);
                }
            }

            #[test]
            fn overflow_and_zero_divisor_panics_match_operator_contracts() {
                let one_raw = Decimal::new(1);
                assert!(std::panic::catch_unwind(|| Decimal::MAX + one_raw).is_err());
                assert!(
                    std::panic::catch_unwind(|| {
                        let mut value = Decimal::MAX;
                        value += one_raw;
                    })
                    .is_err()
                );
                assert!(std::panic::catch_unwind(|| Decimal::ZERO - one_raw).is_err());
                assert!(
                    std::panic::catch_unwind(|| {
                        let mut value = Decimal::ZERO;
                        value -= one_raw;
                    })
                    .is_err()
                );
                assert!(std::panic::catch_unwind(|| Decimal::MAX * Decimal::TWO).is_err());
                assert!(
                    std::panic::catch_unwind(|| {
                        let mut value = Decimal::MAX;
                        value *= Decimal::TWO;
                    })
                    .is_err()
                );

                let zero = Decimal::ZERO;
                assert_eq!(Decimal::ONE.checked_div(zero), None);
                assert!(std::panic::catch_unwind(|| Decimal::ONE / zero).is_err());
                assert!(std::panic::catch_unwind(|| Decimal::ONE.saturating_div(zero)).is_err());
                assert!(std::panic::catch_unwind(|| Decimal::ONE.wrapping_div(zero)).is_err());
                assert!(std::panic::catch_unwind(|| Decimal::ONE.overflowing_div(zero)).is_err());
                assert!(
                    std::panic::catch_unwind(|| {
                        let mut value = Decimal::ONE;
                        value /= zero;
                    })
                    .is_err()
                );

                if $factor != 1 {
                    assert!(std::panic::catch_unwind(|| Decimal::MAX / one_raw).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::MAX;
                            value /= one_raw;
                        })
                        .is_err()
                    );
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
                use crate::common::reference::{ArithmeticOperation, signed_arithmetic_outcome};
                use crate::common::{
                    signed_boundary_values, signed_div_threshold_pairs, signed_mul_threshold_pairs, signed_raw_strategy,
                };
                use decimal64::DecimalI64;
                use num_bigint::BigInt;
                use proptest::prelude::*;

                type Decimal = DecimalI64<$scale>;

                fn assert_truncation_law(numerator: BigInt, denominator: BigInt, quotient: &BigInt) {
                    let remainder = &numerator - quotient * &denominator;
                    assert!(remainder.magnitude() < denominator.magnitude());
                    assert!(remainder == BigInt::from(0_u8) || remainder.sign() == numerator.sign());
                }

                fn assert_add(lhs: i64, rhs: i64) {
                    let lhs_value = Decimal::new(lhs);
                    let rhs_value = Decimal::new(rhs);
                    let expected = signed_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Add).unwrap();
                    assert_eq!(
                        lhs_value.checked_add(rhs_value).map(|value| value.0),
                        expected.checked
                    );
                    assert_eq!(lhs_value.wrapping_add(rhs_value).0, expected.wrapping);
                    assert_eq!(
                        lhs_value.overflowing_add(rhs_value),
                        (Decimal::new(expected.wrapping), expected.overflowing)
                    );
                    assert_eq!(lhs_value.saturating_add(rhs_value).0, expected.saturating);
                    if let Some(expected) = expected.checked {
                        assert_eq!((lhs_value + rhs_value).0, expected);
                        let mut assigned = lhs_value;
                        assigned += rhs_value;
                        assert_eq!(assigned.0, expected);
                    }
                }

                fn assert_sub(lhs: i64, rhs: i64) {
                    let lhs_value = Decimal::new(lhs);
                    let rhs_value = Decimal::new(rhs);
                    let expected = signed_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Sub).unwrap();
                    assert_eq!(
                        lhs_value.checked_sub(rhs_value).map(|value| value.0),
                        expected.checked
                    );
                    assert_eq!(lhs_value.wrapping_sub(rhs_value).0, expected.wrapping);
                    assert_eq!(
                        lhs_value.overflowing_sub(rhs_value),
                        (Decimal::new(expected.wrapping), expected.overflowing)
                    );
                    assert_eq!(lhs_value.saturating_sub(rhs_value).0, expected.saturating);
                    if let Some(expected) = expected.checked {
                        assert_eq!((lhs_value - rhs_value).0, expected);
                        let mut assigned = lhs_value;
                        assigned -= rhs_value;
                        assert_eq!(assigned.0, expected);
                    }
                }

                fn assert_mul(lhs: i64, rhs: i64) {
                    let lhs_value = Decimal::new(lhs);
                    let rhs_value = Decimal::new(rhs);
                    let expected = signed_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Mul).unwrap();
                    assert_eq!(
                        lhs_value.checked_mul(rhs_value).map(|value| value.0),
                        expected.checked
                    );
                    assert_eq!(lhs_value.wrapping_mul(rhs_value).0, expected.wrapping);
                    assert_eq!(
                        lhs_value.overflowing_mul(rhs_value),
                        (Decimal::new(expected.wrapping), expected.overflowing)
                    );
                    assert_eq!(lhs_value.saturating_mul(rhs_value).0, expected.saturating);
                    if let Some(expected) = expected.checked {
                        assert_eq!((lhs_value * rhs_value).0, expected);
                        let mut assigned = lhs_value;
                        assigned *= rhs_value;
                        assert_eq!(assigned.0, expected);
                    }

                    let numerator = BigInt::from(lhs) * BigInt::from(rhs);
                    assert_truncation_law(numerator, BigInt::from($factor), &expected.exact);
                }

                fn assert_div(lhs: i64, rhs: i64) {
                    let lhs_value = Decimal::new(lhs);
                    let rhs_value = Decimal::new(rhs);
                    if rhs == 0 {
                        assert_eq!(lhs_value.checked_div(rhs_value), None);
                        return;
                    }

                    let expected = signed_arithmetic_outcome(lhs, rhs, $factor, ArithmeticOperation::Div).unwrap();
                    assert_eq!(
                        lhs_value.checked_div(rhs_value).map(|value| value.0),
                        expected.checked
                    );
                    assert_eq!(lhs_value.wrapping_div(rhs_value).0, expected.wrapping);
                    assert_eq!(
                        lhs_value.overflowing_div(rhs_value),
                        (Decimal::new(expected.wrapping), expected.overflowing)
                    );
                    assert_eq!(lhs_value.saturating_div(rhs_value).0, expected.saturating);
                    if let Some(expected) = expected.checked {
                        assert_eq!((lhs_value / rhs_value).0, expected);
                        let mut assigned = lhs_value;
                        assigned /= rhs_value;
                        assert_eq!(assigned.0, expected);
                    }

                    let numerator = BigInt::from(lhs) * BigInt::from($factor);
                    assert_truncation_law(numerator, BigInt::from(rhs), &expected.exact);
                }

                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(512))]

                    #[test]
                    fn operation_families_match_bigint_oracles(
                        lhs in signed_raw_strategy($factor),
                        rhs in signed_raw_strategy($factor),
                    ) {
                        assert_add(lhs, rhs);
                        assert_sub(lhs, rhs);
                        assert_mul(lhs, rhs);
                        assert_div(lhs, rhs);
                    }

                    #[test]
                    fn arithmetic_identities_and_unary_operations_hold(raw in signed_raw_strategy($factor)) {
                        let value = Decimal::new(raw);
                        prop_assert_eq!(value.is_zero(), raw == 0);
                        prop_assert_eq!(value.is_positive(), raw > 0);
                        prop_assert_eq!(value.is_negative(), raw < 0);
                        prop_assert_eq!(value.checked_add(Decimal::ZERO), Some(value));
                        prop_assert_eq!(value.checked_sub(Decimal::ZERO), Some(value));
                        prop_assert_eq!(value.checked_mul(Decimal::ONE), Some(value));
                        prop_assert_eq!(value.checked_div(Decimal::ONE), Some(value));
                        prop_assert_eq!(Decimal::ZERO.checked_mul(value), Some(Decimal::ZERO));
                        prop_assert_eq!(value.checked_neg(), raw.checked_neg().map(Decimal::new));
                        prop_assert_eq!(value.checked_abs(), raw.checked_abs().map(Decimal::new));
                        prop_assert_eq!(value.unsigned_abs().0, raw.unsigned_abs());
                    }
                }

                #[test]
                fn deterministic_boundary_matrix_matches_bigint_oracles() {
                    let values = signed_boundary_values($factor);
                    for &lhs in &values {
                        for &rhs in &values {
                            assert_add(lhs, rhs);
                            assert_sub(lhs, rhs);
                            assert_mul(lhs, rhs);
                            assert_div(lhs, rhs);
                        }
                    }

                    for (lhs, rhs) in signed_mul_threshold_pairs($factor) {
                        assert_mul(lhs, rhs);
                    }
                    for (lhs, rhs) in signed_div_threshold_pairs($factor) {
                        assert_div(lhs, rhs);
                    }
                }

                #[test]
                fn overflow_and_zero_divisor_panics_match_operator_contracts() {
                    let one_raw = Decimal::new(1);
                    assert!(std::panic::catch_unwind(|| Decimal::MAX + one_raw).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::MAX;
                            value += one_raw;
                        })
                        .is_err()
                    );
                    assert!(std::panic::catch_unwind(|| Decimal::MIN - one_raw).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::MIN;
                            value -= one_raw;
                        })
                        .is_err()
                    );
                    assert!(std::panic::catch_unwind(|| Decimal::MAX * Decimal::TWO).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::MAX;
                            value *= Decimal::TWO;
                        })
                        .is_err()
                    );

                    let zero = Decimal::ZERO;
                    assert_eq!(Decimal::ONE.checked_div(zero), None);
                    assert!(std::panic::catch_unwind(|| Decimal::ONE / zero).is_err());
                    assert!(std::panic::catch_unwind(|| Decimal::ONE.saturating_div(zero)).is_err());
                    assert!(std::panic::catch_unwind(|| Decimal::ONE.wrapping_div(zero)).is_err());
                    assert!(std::panic::catch_unwind(|| Decimal::ONE.overflowing_div(zero)).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::ONE;
                            value /= zero;
                        })
                        .is_err()
                    );

                    let divisor = if $factor == 1 { Decimal::new(-1) } else { one_raw };
                    assert!(std::panic::catch_unwind(|| Decimal::MIN / divisor).is_err());
                    assert!(
                        std::panic::catch_unwind(|| {
                            let mut value = Decimal::MIN;
                            value /= divisor;
                        })
                        .is_err()
                    );
                }
            }
        };
    }

    for_each_scale!(suite);
}
