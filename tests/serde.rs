#![cfg(feature = "serde")]

mod common;

use common::for_each_scale;
use decimal64::{DecimalU64, U2, U8};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Deserialize, Serialize, Debug)]
struct Item {
    one: DecimalU64<U8>,
    two: DecimalU64<U8>,
    three: DecimalU64<U8>,
    four: DecimalU64<U8>,
}

#[test]
fn should_deserialize() {
    let item: Item = serde_json::from_reader(File::open("tests/item.json").unwrap()).unwrap();
    assert_eq!("123.45000000", item.one.to_string());
    assert_eq!("456.78000000", item.two.to_string());
    assert_eq!("100.00000000", item.three.to_string());
    assert_eq!("0.50000000", item.four.to_string());
}

#[test]
fn should_serialize_as_a_string_without_losing_scale() {
    let value = DecimalU64::<U8>::from_str("12.34").unwrap();
    assert_eq!(r#""12.34000000""#, serde_json::to_string(&value).unwrap());
}

#[test]
fn should_parse_string_input_exactly() {
    let value: DecimalU64<U2> = serde_json::from_str(r#""12.34""#).unwrap();
    assert_eq!("12.34", value.to_string());
    assert!(serde_json::from_str::<DecimalU64<U2>>(r#""12.345""#).is_err());
}

#[test]
fn should_reject_numeric_input() {
    for input in ["12.345", "100", "-1"] {
        assert!(serde_json::from_str::<DecimalU64<U8>>(input).is_err());
    }
}

#[test]
fn should_reject_string_overflow_at_the_target_scale() {
    assert!(serde_json::from_str::<DecimalU64<U8>>(r#""18446744073709551615""#).is_err());
}

macro_rules! serde_suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use super::*;
            use crate::common::reference::format_fixed;

            type Decimal = DecimalU64<$scale>;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(256))]

                #[test]
                fn string_representation_round_trips_without_losing_scale(raw in any::<u64>()) {
                    let value = Decimal::new(raw);
                    let expected = format_fixed(raw, $digits, $factor);
                    let serialized = serde_json::to_string(&value).unwrap();

                    prop_assert_eq!(&serialized, &serde_json::to_string(&expected).unwrap());
                    prop_assert_eq!(serde_json::from_str::<Decimal>(&serialized).unwrap(), value);
                }
            }
        }
    };
}

for_each_scale!(serde_suite);
