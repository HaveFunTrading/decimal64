use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use crate::common::reference::{format_fixed, format_trimmed};
            use decimal64::{DecimalU64, Error, InvalidInputKind, ScaleMetrics};
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            #[test]
            fn scale_metadata_matches_the_reference() {
                assert_eq!(<$scale>::SCALE, $digits);
                assert_eq!(<$scale>::SCALE_FACTOR, $factor);
                assert_eq!(<$scale>::REQUIRED_BUFFER_LEN, $max_len);
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn formatting_matches_the_reference_and_round_trips(raw in any::<u64>()) {
                    let value = Decimal::new(raw);
                    let fixed = format_fixed(raw, $digits, $factor);
                    let trimmed = format_trimmed(raw, $digits, $factor);

                    prop_assert_eq!(value.to_string(), fixed.as_str());
                    prop_assert_eq!(Decimal::from_str(&fixed), Ok(value));
                    prop_assert_eq!(Decimal::from_str(&trimmed), Ok(value));

                    let mut fixed_buffer = vec![0xa5; $max_len];
                    let fixed_len = value.try_write_to(&mut fixed_buffer).unwrap();
                    prop_assert_eq!(&fixed_buffer[..fixed_len], fixed.as_bytes());
                    prop_assert!(fixed_buffer[fixed_len..].iter().all(|byte| *byte == 0xa5));

                    let mut trimmed_buffer = vec![0xa5; $max_len];
                    let trimmed_len = value.try_write_to_trimmed(&mut trimmed_buffer).unwrap();
                    prop_assert_eq!(&trimmed_buffer[..trimmed_len], trimmed.as_bytes());
                    prop_assert!(trimmed_buffer[trimmed_len..].iter().all(|byte| *byte == 0xa5));
                }

                #[test]
                fn fallible_writers_report_the_actual_required_length_without_writing(raw in any::<u64>()) {
                    let value = Decimal::new(raw);
                    let fixed_len = format_fixed(raw, $digits, $factor).len();
                    let trimmed_len = format_trimmed(raw, $digits, $factor).len();

                    let mut fixed_buffer = vec![0xa5; fixed_len - 1];
                    let fixed_before = fixed_buffer.clone();
                    let fixed_error = value.try_write_to(&mut fixed_buffer).unwrap_err();
                    prop_assert_eq!(fixed_error.required, fixed_len);
                    prop_assert_eq!(fixed_error.provided, fixed_len - 1);
                    prop_assert_eq!(fixed_buffer, fixed_before);

                    let mut trimmed_buffer = vec![0xa5; trimmed_len - 1];
                    let trimmed_before = trimmed_buffer.clone();
                    let trimmed_error = value.try_write_to_trimmed(&mut trimmed_buffer).unwrap_err();
                    prop_assert_eq!(trimmed_error.required, trimmed_len);
                    prop_assert_eq!(trimmed_error.provided, trimmed_len - 1);
                    prop_assert_eq!(trimmed_buffer, trimmed_before);
                }

                #[test]
                fn exact_parser_accepts_redundant_zeroes_and_rejects_lost_precision(raw in any::<u64>()) {
                    let value = Decimal::new(raw);
                    let fixed = format_fixed(raw, $digits, $factor);
                    let with_zeroes = if $digits == 0 {
                        format!("+000{fixed}.000")
                    } else {
                        format!("+000{fixed}000")
                    };
                    prop_assert_eq!(Decimal::from_str(&with_zeroes), Ok(value));

                    let inexact = if $digits == 0 {
                        format!("{fixed}.1")
                    } else {
                        format!("{fixed}1")
                    };
                    prop_assert_eq!(
                        Decimal::from_str(&inexact),
                        Err(Error::InvalidInput(InvalidInputKind::PrecisionLoss))
                    );
                }

                #[test]
                fn byte_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
                    let _ = Decimal::from_slice(&bytes);
                }
            }
        }
    };
}

for_each_scale!(suite);
