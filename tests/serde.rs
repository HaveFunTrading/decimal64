use decimal::{DecimalU64, U8};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fs::File;

#[cfg(feature = "serde")]
#[test]
fn should_deserialize() {
    #[derive(Deserialize, Serialize, Debug)]
    struct Item {
        one: DecimalU64<U8>,
        two: DecimalU64<U8>,
        three: DecimalU64<U8>,
        four: DecimalU64<U8>,
    }

    let item: Item = serde_json::from_reader(File::open("tests/item.json").unwrap()).unwrap();
    assert_eq!("123.45000000", item.one.to_string());
    assert_eq!("456.78000000", item.two.to_string());
    assert_eq!("100.00000000", item.three.to_string());
    assert_eq!("0.50000000", item.four.to_string());
}
