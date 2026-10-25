#![cfg(feature = "serde")]

use decimal64::{DecimalU64, U2, U8};
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
fn should_round_floating_point_input_at_the_target_scale() {
    let value: DecimalU64<U2> = serde_json::from_str("12.345").unwrap();
    assert_eq!("12.35", value.to_string());
}

#[test]
fn should_reject_negative_numeric_input() {
    let error = serde_json::from_str::<DecimalU64<U8>>("-1").unwrap_err();
    assert!(error.to_string().contains("negative number"));
}

#[test]
fn should_reject_integer_overflow_at_the_target_scale() {
    assert!(serde_json::from_str::<DecimalU64<U8>>("18446744073709551615").is_err());
}
