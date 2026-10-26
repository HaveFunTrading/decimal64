use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use decimal64::DecimalU64;
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn operation_families_match_independent_integer_oracles(lhs in any::<u64>(), rhs in any::<u64>()) {
                    let lhs_value = Decimal::new(lhs);
                    let rhs_value = Decimal::new(rhs);

                    let (add, add_overflowed) = lhs.overflowing_add(rhs);
                    prop_assert_eq!(lhs_value.checked_add(rhs_value), (!add_overflowed).then(|| Decimal::new(add)));
                    prop_assert_eq!(lhs_value.wrapping_add(rhs_value), Decimal::new(add));
                    prop_assert_eq!(lhs_value.overflowing_add(rhs_value), (Decimal::new(add), add_overflowed));
                    prop_assert_eq!(lhs_value.saturating_add(rhs_value), Decimal::new(lhs.saturating_add(rhs)));

                    let (sub, sub_overflowed) = lhs.overflowing_sub(rhs);
                    prop_assert_eq!(lhs_value.checked_sub(rhs_value), (!sub_overflowed).then(|| Decimal::new(sub)));
                    prop_assert_eq!(lhs_value.wrapping_sub(rhs_value), Decimal::new(sub));
                    prop_assert_eq!(lhs_value.overflowing_sub(rhs_value), (Decimal::new(sub), sub_overflowed));
                    prop_assert_eq!(lhs_value.saturating_sub(rhs_value), Decimal::new(lhs.saturating_sub(rhs)));

                    let product = lhs as u128 * rhs as u128 / $factor as u128;
                    let mul_overflowed = product > u64::MAX as u128;
                    let wrapped_product = product as u64;
                    prop_assert_eq!(
                        lhs_value.checked_mul(rhs_value),
                        (!mul_overflowed).then(|| Decimal::new(wrapped_product))
                    );
                    prop_assert_eq!(lhs_value.wrapping_mul(rhs_value), Decimal::new(wrapped_product));
                    prop_assert_eq!(
                        lhs_value.overflowing_mul(rhs_value),
                        (Decimal::new(wrapped_product), mul_overflowed)
                    );
                    prop_assert_eq!(
                        lhs_value.saturating_mul(rhs_value),
                        if mul_overflowed { Decimal::MAX } else { Decimal::new(wrapped_product) }
                    );

                    if rhs == 0 {
                        prop_assert_eq!(lhs_value.checked_div(rhs_value), None);
                    } else {
                        let quotient = lhs as u128 * $factor as u128 / rhs as u128;
                        let div_overflowed = quotient > u64::MAX as u128;
                        let wrapped_quotient = quotient as u64;
                        prop_assert_eq!(
                            lhs_value.checked_div(rhs_value),
                            (!div_overflowed).then(|| Decimal::new(wrapped_quotient))
                        );
                        prop_assert_eq!(lhs_value.wrapping_div(rhs_value), Decimal::new(wrapped_quotient));
                        prop_assert_eq!(
                            lhs_value.overflowing_div(rhs_value),
                            (Decimal::new(wrapped_quotient), div_overflowed)
                        );
                        prop_assert_eq!(
                            lhs_value.saturating_div(rhs_value),
                            if div_overflowed { Decimal::MAX } else { Decimal::new(wrapped_quotient) }
                        );
                    }
                }

                #[test]
                fn arithmetic_identities_hold(raw in any::<u64>()) {
                    let value = Decimal::new(raw);
                    prop_assert_eq!(value.is_zero(), raw == 0);
                    prop_assert_eq!(value.checked_add(Decimal::ZERO), Some(value));
                    prop_assert_eq!(value.checked_sub(Decimal::ZERO), Some(value));
                    prop_assert_eq!(value.checked_mul(Decimal::ONE), Some(value));
                    prop_assert_eq!(value.checked_div(Decimal::ONE), Some(value));
                    prop_assert_eq!(Decimal::ZERO.checked_mul(value), Some(Decimal::ZERO));
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
                use proptest::prelude::*;

                type Decimal = DecimalI64<$scale>;

                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(512))]

                    #[test]
                    fn operation_families_match_independent_integer_oracles(lhs in any::<i64>(), rhs in any::<i64>()) {
                        let lhs_value = Decimal::new(lhs);
                        let rhs_value = Decimal::new(rhs);

                        let (add, add_overflowed) = lhs.overflowing_add(rhs);
                        prop_assert_eq!(lhs_value.checked_add(rhs_value), lhs.checked_add(rhs).map(Decimal::new));
                        prop_assert_eq!(lhs_value.wrapping_add(rhs_value), Decimal::new(add));
                        prop_assert_eq!(lhs_value.overflowing_add(rhs_value), (Decimal::new(add), add_overflowed));
                        prop_assert_eq!(lhs_value.saturating_add(rhs_value), Decimal::new(lhs.saturating_add(rhs)));

                        let (sub, sub_overflowed) = lhs.overflowing_sub(rhs);
                        prop_assert_eq!(lhs_value.checked_sub(rhs_value), lhs.checked_sub(rhs).map(Decimal::new));
                        prop_assert_eq!(lhs_value.wrapping_sub(rhs_value), Decimal::new(sub));
                        prop_assert_eq!(lhs_value.overflowing_sub(rhs_value), (Decimal::new(sub), sub_overflowed));
                        prop_assert_eq!(lhs_value.saturating_sub(rhs_value), Decimal::new(lhs.saturating_sub(rhs)));

                        let product = lhs as i128 * rhs as i128 / $factor as i128;
                        let mul_overflowed = product < i64::MIN as i128 || product > i64::MAX as i128;
                        prop_assert_eq!(
                            lhs_value.checked_mul(rhs_value),
                            (!mul_overflowed).then(|| Decimal::new(product as i64))
                        );
                        prop_assert_eq!(lhs_value.wrapping_mul(rhs_value), Decimal::new(product as i64));
                        prop_assert_eq!(
                            lhs_value.overflowing_mul(rhs_value),
                            (Decimal::new(product as i64), mul_overflowed)
                        );
                        prop_assert_eq!(
                            lhs_value.saturating_mul(rhs_value),
                            Decimal::new(product.clamp(i64::MIN as i128, i64::MAX as i128) as i64)
                        );

                        if rhs == 0 {
                            prop_assert_eq!(lhs_value.checked_div(rhs_value), None);
                        } else {
                            let quotient = lhs as i128 * $factor as i128 / rhs as i128;
                            let div_overflowed = quotient < i64::MIN as i128 || quotient > i64::MAX as i128;
                            prop_assert_eq!(
                                lhs_value.checked_div(rhs_value),
                                (!div_overflowed).then(|| Decimal::new(quotient as i64))
                            );
                            prop_assert_eq!(lhs_value.wrapping_div(rhs_value), Decimal::new(quotient as i64));
                            prop_assert_eq!(
                                lhs_value.overflowing_div(rhs_value),
                                (Decimal::new(quotient as i64), div_overflowed)
                            );
                            prop_assert_eq!(
                                lhs_value.saturating_div(rhs_value),
                                Decimal::new(quotient.clamp(i64::MIN as i128, i64::MAX as i128) as i64)
                            );
                        }
                    }

                    #[test]
                    fn arithmetic_identities_and_unary_operations_hold(raw in any::<i64>()) {
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
            }
        };
    }

    for_each_scale!(suite);
}
