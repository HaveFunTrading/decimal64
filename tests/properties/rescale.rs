use crate::common::for_each_scale;
use decimal64::{DecimalU64, Error, ScaleMetrics};
use proptest::prelude::*;
use proptest::test_runner::TestCaseResult;
use std::fmt::Debug;

fn power_of_ten(exponent: u8) -> u128 {
    (0..exponent).fold(1_u128, |value, _| value * 10)
}

fn check_pair<S, T>(raw: u64, source_scale: u8, target_scale: u8) -> TestCaseResult
where
    S: ScaleMetrics,
    T: ScaleMetrics + Debug + PartialEq,
{
    let actual = DecimalU64::<S>::new(raw).rescale::<T>();
    let expected = if target_scale >= source_scale {
        let factor = power_of_ten(target_scale - source_scale);
        let scaled = raw as u128 * factor;
        if scaled > u64::MAX as u128 {
            Err(Error::Overflow)
        } else {
            Ok(DecimalU64::<T>::new(scaled as u64))
        }
    } else {
        let divisor = power_of_ten(source_scale - target_scale);
        let quotient = raw as u128 / divisor;
        let remainder = raw as u128 % divisor;
        let rounded = quotient + u128::from(remainder >= divisor / 2);
        Ok(DecimalU64::<T>::new(rounded as u64))
    };

    prop_assert_eq!(actual, expected);
    Ok(())
}

macro_rules! check_targets {
    ($raw:expr, $source:ty, $source_scale:expr) => {{
        check_pair::<$source, decimal64::U0>($raw, $source_scale, 0)?;
        check_pair::<$source, decimal64::U1>($raw, $source_scale, 1)?;
        check_pair::<$source, decimal64::U2>($raw, $source_scale, 2)?;
        check_pair::<$source, decimal64::U3>($raw, $source_scale, 3)?;
        check_pair::<$source, decimal64::U4>($raw, $source_scale, 4)?;
        check_pair::<$source, decimal64::U5>($raw, $source_scale, 5)?;
        check_pair::<$source, decimal64::U6>($raw, $source_scale, 6)?;
        check_pair::<$source, decimal64::U7>($raw, $source_scale, 7)?;
        check_pair::<$source, decimal64::U8>($raw, $source_scale, 8)?;
    }};
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_024))]

    #[test]
    fn every_scale_pair_matches_the_reference(raw in any::<u64>()) {
        macro_rules! check_source {
            ($module:ident, $source:ty, $source_scale:expr, $factor:expr, $max_len:expr) => {
                check_targets!(raw, $source, $source_scale);
            };
        }

        for_each_scale!(check_source);
    }
}

mod signed {
    use crate::common::for_each_scale;
    use decimal64::{DecimalI64, Error, ScaleMetrics};
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;
    use std::fmt::Debug;

    fn check_pair<S, T>(raw: i64, source_scale: u8, target_scale: u8) -> TestCaseResult
    where
        S: ScaleMetrics,
        T: ScaleMetrics + Debug + PartialEq,
    {
        let actual = DecimalI64::<S>::new(raw).rescale::<T>();
        let difference = source_scale.abs_diff(target_scale);
        let factor = 10_i128.pow(difference as u32);
        let expected = if target_scale >= source_scale {
            let scaled = raw as i128 * factor;
            if scaled < i64::MIN as i128 || scaled > i64::MAX as i128 {
                Err(Error::Overflow)
            } else {
                Ok(DecimalI64::<T>::new(scaled as i64))
            }
        } else {
            let quotient = raw as i128 / factor;
            let remainder = raw as i128 % factor;
            let rounded = if remainder.unsigned_abs() >= factor as u128 / 2 {
                quotient + if raw < 0 { -1 } else { 1 }
            } else {
                quotient
            };
            Ok(DecimalI64::<T>::new(rounded as i64))
        };

        prop_assert_eq!(actual, expected);
        Ok(())
    }

    macro_rules! check_targets {
        ($raw:expr, $source:ty, $source_scale:expr) => {{
            check_pair::<$source, decimal64::U0>($raw, $source_scale, 0)?;
            check_pair::<$source, decimal64::U1>($raw, $source_scale, 1)?;
            check_pair::<$source, decimal64::U2>($raw, $source_scale, 2)?;
            check_pair::<$source, decimal64::U3>($raw, $source_scale, 3)?;
            check_pair::<$source, decimal64::U4>($raw, $source_scale, 4)?;
            check_pair::<$source, decimal64::U5>($raw, $source_scale, 5)?;
            check_pair::<$source, decimal64::U6>($raw, $source_scale, 6)?;
            check_pair::<$source, decimal64::U7>($raw, $source_scale, 7)?;
            check_pair::<$source, decimal64::U8>($raw, $source_scale, 8)?;
        }};
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1_024))]

        #[test]
        fn every_scale_pair_matches_the_reference(raw in any::<i64>()) {
            macro_rules! check_source {
                ($module:ident, $source:ty, $source_scale:expr, $factor:expr, $max_len:expr) => {
                    check_targets!(raw, $source, $source_scale);
                };
            }

            for_each_scale!(check_source);
        }
    }
}
