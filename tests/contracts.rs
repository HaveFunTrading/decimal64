use decimal64::round::{Ceil, Floor, HalfUp};
use decimal64::{DecimalI64, DecimalU64, Error, InvalidInputKind, U0, U2, U3, U4, U8};

mod representation {
    use super::*;

    #[test]
    fn both_types_are_transparent_eight_byte_values() {
        assert_eq!(size_of::<DecimalU64<U0>>(), size_of::<u64>());
        assert_eq!(align_of::<DecimalU64<U8>>(), align_of::<u64>());
        assert_eq!(size_of::<DecimalI64<U0>>(), size_of::<i64>());
        assert_eq!(align_of::<DecimalI64<U8>>(), align_of::<i64>());
    }

    #[test]
    fn defaults_constants_and_raw_construction_are_consistent() {
        assert_eq!(DecimalU64::<U2>::default(), DecimalU64::ZERO);
        assert_eq!(DecimalI64::<U2>::default(), DecimalI64::ZERO);
        assert_eq!(DecimalU64::<U2>::ONE.0, 100);
        assert_eq!(DecimalI64::<U2>::ONE.0, 100);
        assert_eq!(DecimalU64::<U2>::TEN.0, 1_000);
        assert_eq!(DecimalI64::<U2>::TEN.0, 1_000);
        assert_eq!(DecimalU64::<U2>::new(123).0, 123);
        assert_eq!(DecimalI64::<U2>::new(-123).0, -123);
    }

    #[test]
    fn ordering_is_the_raw_integer_order_at_a_fixed_scale() {
        assert!(DecimalU64::<U2>::new(1) < DecimalU64::new(2));
        assert!(DecimalI64::<U2>::new(-2) < DecimalI64::new(-1));
        assert_eq!(DecimalI64::<U2>::new(-1), DecimalI64::new(-1));
    }

    #[test]
    fn sign_and_zero_predicates_match_integer_semantics() {
        assert!(DecimalU64::<U2>::ZERO.is_zero());
        assert!(!DecimalU64::<U2>::new(1).is_zero());

        assert!(DecimalI64::<U2>::ZERO.is_zero());
        assert!(!DecimalI64::<U2>::ZERO.is_positive());
        assert!(!DecimalI64::<U2>::ZERO.is_negative());
        assert!(DecimalI64::<U2>::new(1).is_positive());
        assert!(!DecimalI64::<U2>::new(1).is_negative());
        assert!(DecimalI64::<U2>::new(-1).is_negative());
        assert!(!DecimalI64::<U2>::new(-1).is_positive());
    }

    #[test]
    fn split_uses_the_documented_signedness() {
        assert_eq!(DecimalU64::<U2>::new(1_234).split(), (12, 34));
        assert_eq!(DecimalI64::<U2>::new(1_234).split(), (12, 34));
        assert_eq!(DecimalI64::<U2>::new(-1_234).split(), (-12, -34));
    }
}

mod parsing {
    use super::*;

    #[test]
    fn parses_unsigned_and_signed_boundaries() {
        assert_eq!(DecimalU64::<U0>::from_str("18446744073709551615"), Ok(DecimalU64::MAX));
        assert_eq!(DecimalI64::<U0>::from_str("9223372036854775807"), Ok(DecimalI64::MAX));
        assert_eq!(DecimalI64::<U0>::from_str("-9223372036854775808"), Ok(DecimalI64::MIN));
        assert_eq!(DecimalI64::<U2>::from_str("-0.00"), Ok(DecimalI64::ZERO));

        assert_eq!(DecimalU64::<U0>::from_str("18446744073709551616"), Err(Error::Overflow));
        assert_eq!(DecimalI64::<U0>::from_str("9223372036854775808"), Err(Error::Overflow));
        assert_eq!(DecimalI64::<U0>::from_str("-9223372036854775809"), Err(Error::Overflow));
    }

    #[test]
    fn parsing_is_exact_for_both_types() {
        assert_eq!(DecimalU64::<U2>::from_str("1.2300"), Ok(DecimalU64::new(123)));
        assert_eq!(DecimalI64::<U2>::from_str("-1.2300"), Ok(DecimalI64::new(-123)));
        assert_eq!(
            DecimalU64::<U2>::from_str("1.2301"),
            Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss))
        );
        assert_eq!(
            DecimalI64::<U2>::from_str("-1.2301"),
            Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss))
        );
    }

    #[test]
    fn malformed_inputs_are_rejected_consistently() {
        for input in ["", ".", "+", "1.2.3", "1e2", " 1", "1 "] {
            assert!(
                DecimalU64::<U2>::from_str(input).is_err(),
                "unsigned accepted {input:?}"
            );
            assert!(DecimalI64::<U2>::from_str(input).is_err(), "signed accepted {input:?}");
        }
        assert_eq!(
            DecimalU64::<U2>::from_str("-1"),
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
        );
    }

    #[test]
    fn std_from_str_matches_the_inherent_parser() {
        assert_eq!("12.34".parse::<DecimalU64<U2>>(), DecimalU64::from_str("12.34"));
        assert_eq!("-12.34".parse::<DecimalI64<U2>>(), DecimalI64::from_str("-12.34"));
    }
}

mod formatting {
    use super::*;

    #[test]
    fn fixed_and_trimmed_formatting_are_symmetric() {
        let unsigned = DecimalU64::<U4>::from_str("12.3400").unwrap();
        let signed = DecimalI64::<U4>::from_str("-12.3400").unwrap();
        assert_eq!(unsigned.to_string(), "12.3400");
        assert_eq!(signed.to_string(), "-12.3400");

        let mut unsigned_buffer = [0; 16];
        let unsigned_len = unsigned.write_to_trimmed(&mut unsigned_buffer);
        assert_eq!(&unsigned_buffer[..unsigned_len], b"12.34");

        let mut signed_buffer = [0; 16];
        let signed_len = signed.write_to_trimmed(&mut signed_buffer);
        assert_eq!(&signed_buffer[..signed_len], b"-12.34");
    }

    #[test]
    fn fallible_writers_do_not_modify_small_buffers() {
        let mut unsigned_buffer = [0xa5; 4];
        let unsigned_error = DecimalU64::<U2>::new(1_234)
            .try_write_to(&mut unsigned_buffer)
            .unwrap_err();
        assert_eq!(unsigned_error.required, 5);
        assert_eq!(unsigned_error.provided, 4);
        assert_eq!(unsigned_buffer, [0xa5; 4]);

        let mut signed_buffer = [0xa5; 5];
        let signed_error = DecimalI64::<U2>::new(-1_234)
            .try_write_to(&mut signed_buffer)
            .unwrap_err();
        assert_eq!(signed_error.required, 6);
        assert_eq!(signed_error.provided, 5);
        assert_eq!(signed_buffer, [0xa5; 5]);
    }

    #[test]
    fn minimum_signed_value_formats_without_overflow() {
        assert_eq!(DecimalI64::<U0>::MIN.to_string(), "-9223372036854775808");
        assert_eq!(DecimalI64::<U8>::MIN.to_string(), "-92233720368.54775808");
    }
}

mod conversion {
    use super::*;

    #[test]
    fn floating_point_rounding_is_symmetric() {
        assert_eq!(DecimalU64::<U2>::from_f64(0.125), Ok(DecimalU64::new(13)));
        assert_eq!(DecimalI64::<U2>::from_f64(0.125), Ok(DecimalI64::new(13)));
        assert_eq!(DecimalI64::<U2>::from_f64(-0.125), Ok(DecimalI64::new(-13)));
        assert_eq!(DecimalI64::<U2>::from_f64(-0.0), Ok(DecimalI64::ZERO));
    }

    #[test]
    fn floating_point_special_values_are_rejected() {
        for result in [
            DecimalI64::<U2>::from_f64(f64::INFINITY),
            DecimalI64::<U2>::from_f64(f64::NEG_INFINITY),
        ] {
            assert_eq!(result, Err(Error::InvalidInput(InvalidInputKind::InfiniteNumber)));
        }
        assert_eq!(
            DecimalI64::<U2>::from_f64(f64::NAN),
            Err(Error::InvalidInput(InvalidInputKind::NotANumber))
        );
        assert_eq!(
            DecimalU64::<U2>::from_f64(-1.0),
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
        );
    }

    #[test]
    fn signedness_conversions_check_range_and_sign() {
        assert_eq!(
            DecimalI64::<U2>::try_from(DecimalU64::new(123)),
            Ok(DecimalI64::new(123))
        );
        assert_eq!(
            DecimalI64::<U2>::try_from(DecimalU64::new(i64::MAX as u64 + 1)),
            Err(Error::Overflow)
        );
        assert_eq!(
            DecimalU64::<U2>::try_from(DecimalI64::new(123)),
            Ok(DecimalU64::new(123))
        );
        assert_eq!(
            DecimalU64::<U2>::try_from(DecimalI64::new(-1)),
            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
        );
    }

    #[test]
    fn rescale_rounds_halfway_values_away_from_zero() {
        assert_eq!(DecimalU64::<U3>::new(1_235).rescale::<U2>(), Ok(DecimalU64::new(124)));
        assert_eq!(DecimalI64::<U3>::new(1_235).rescale::<U2>(), Ok(DecimalI64::new(124)));
        assert_eq!(DecimalI64::<U3>::new(-1_235).rescale::<U2>(), Ok(DecimalI64::new(-124)));
        assert_eq!(DecimalI64::<U3>::new(-1_234).rescale::<U2>(), Ok(DecimalI64::new(-123)));
    }
}

mod arithmetic {
    use super::*;

    #[test]
    fn standard_operators_retain_scale_and_truncate_mul_and_div() {
        let unsigned = DecimalU64::<U2>::new(125);
        assert_eq!(unsigned * DecimalU64::new(250), DecimalU64::new(312));
        assert_eq!(unsigned / DecimalU64::new(200), DecimalU64::new(62));

        let signed = DecimalI64::<U2>::new(-125);
        assert_eq!(signed * DecimalI64::new(250), DecimalI64::new(-312));
        assert_eq!(signed / DecimalI64::new(200), DecimalI64::new(-62));
    }

    #[test]
    fn operation_families_handle_numeric_boundaries() {
        assert_eq!(DecimalU64::<U2>::MAX.checked_add(DecimalU64::new(1)), None);
        assert_eq!(
            DecimalU64::<U2>::MAX.saturating_add(DecimalU64::new(1)),
            DecimalU64::MAX
        );
        assert_eq!(DecimalU64::<U2>::MAX.wrapping_add(DecimalU64::new(1)), DecimalU64::ZERO);

        assert_eq!(DecimalI64::<U2>::MAX.checked_add(DecimalI64::new(1)), None);
        assert_eq!(DecimalI64::<U2>::MIN.checked_sub(DecimalI64::new(1)), None);
        assert_eq!(
            DecimalI64::<U2>::MAX.saturating_add(DecimalI64::new(1)),
            DecimalI64::MAX
        );
        assert_eq!(
            DecimalI64::<U2>::MIN.saturating_sub(DecimalI64::new(1)),
            DecimalI64::MIN
        );
        assert_eq!(
            DecimalI64::<U2>::MAX.overflowing_add(DecimalI64::new(1)),
            (DecimalI64::MIN, true)
        );
    }

    #[test]
    fn signed_negation_and_absolute_value_match_i64_contracts() {
        let value = DecimalI64::<U2>::new(-123);
        assert_eq!(-value, DecimalI64::new(123));
        assert_eq!(value.abs(), DecimalI64::new(123));
        assert_eq!(value.unsigned_abs(), DecimalU64::new(123));
        assert_eq!(DecimalI64::<U2>::MIN.checked_neg(), None);
        assert_eq!(DecimalI64::<U2>::MIN.checked_abs(), None);
        assert!(std::panic::catch_unwind(|| DecimalI64::<U2>::MIN.abs()).is_err());
        assert_eq!(DecimalI64::<U2>::MIN.saturating_abs(), DecimalI64::MAX);
        assert_eq!(DecimalI64::<U2>::MIN.wrapping_abs(), DecimalI64::MIN);
        assert_eq!(DecimalI64::<U2>::MIN.overflowing_abs(), (DecimalI64::MIN, true));
    }

    #[test]
    fn assignments_and_iterators_work_for_both_types() {
        let mut unsigned = DecimalU64::<U2>::ONE;
        unsigned += DecimalU64::TWO;
        unsigned *= &DecimalU64::TWO;
        assert_eq!(unsigned, DecimalU64::new(600));
        let unsigned_sum: DecimalU64<U2> = [DecimalU64::ONE, DecimalU64::TWO].into_iter().sum();
        let unsigned_product: DecimalU64<U2> = [DecimalU64::TWO, DecimalU64::THREE].into_iter().product();
        assert_eq!(unsigned_sum, DecimalU64::new(300));
        assert_eq!(unsigned_product, DecimalU64::new(600));

        let mut signed = DecimalI64::<U2>::ONE;
        signed -= DecimalI64::TWO;
        signed *= &DecimalI64::TWO;
        assert_eq!(signed, DecimalI64::new(-200));
        let signed_sum: DecimalI64<U2> = [DecimalI64::new(-100), DecimalI64::TWO].into_iter().sum();
        let signed_product: DecimalI64<U2> = [DecimalI64::new(-200), DecimalI64::THREE].into_iter().product();
        assert_eq!(signed_sum, DecimalI64::new(100));
        assert_eq!(signed_product, DecimalI64::new(-600));
    }

    #[test]
    fn checked_division_handles_zero_for_both_types() {
        assert_eq!(DecimalU64::<U2>::ONE.checked_div(DecimalU64::ZERO), None);
        assert_eq!(DecimalI64::<U2>::ONE.checked_div(DecimalI64::ZERO), None);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn unsigned_operator_panics_on_overflow() {
        let _ = DecimalU64::<U2>::MAX + DecimalU64::new(1);
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn signed_operator_panics_on_overflow() {
        let _ = -DecimalI64::<U2>::MIN;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn signed_division_operator_panics_on_zero() {
        let _ = DecimalI64::<U2>::ONE / DecimalI64::ZERO;
    }
}

mod rounding {
    use super::*;

    #[test]
    fn signed_rounding_uses_mathematical_directions() {
        let tick = DecimalI64::<U2>::new(10);
        let value = DecimalI64::<U2>::new(-125);
        assert_eq!(value.round::<HalfUp>(tick), DecimalI64::new(-130));
        assert_eq!(value.round::<Floor>(tick), DecimalI64::new(-130));
        assert_eq!(value.round::<Ceil>(tick), DecimalI64::new(-120));
    }

    #[test]
    fn unsigned_rounding_preserves_existing_semantics() {
        let tick = DecimalU64::<U2>::new(10);
        let value = DecimalU64::<U2>::new(125);
        assert_eq!(value.round::<HalfUp>(tick), DecimalU64::new(130));
        assert_eq!(value.round::<Floor>(tick), DecimalU64::new(120));
        assert_eq!(value.round::<Ceil>(tick), DecimalU64::new(130));
    }

    #[test]
    #[should_panic(expected = "attempt to round with a non-positive tick size")]
    fn signed_rounding_rejects_zero_ticks() {
        let _ = DecimalI64::<U2>::ONE.round::<Floor>(DecimalI64::ZERO);
    }

    #[test]
    #[should_panic(expected = "attempt to round with a non-positive tick size")]
    fn signed_rounding_rejects_negative_ticks() {
        let _ = DecimalI64::<U2>::ONE.round::<Floor>(DecimalI64::new(-1));
    }

    #[test]
    #[should_panic(expected = "attempt to round with overflow")]
    fn signed_floor_panics_when_the_result_is_below_minimum() {
        let _ = DecimalI64::<U0>::MIN.round::<Floor>(DecimalI64::new(10));
    }

    #[test]
    #[should_panic(expected = "attempt to round with overflow")]
    fn signed_ceil_panics_when_the_result_is_above_maximum() {
        let _ = DecimalI64::<U0>::MAX.round::<Ceil>(DecimalI64::new(10));
    }
}
